#![feature(box_syntax, box_patterns)]

#[macro_use]
extern crate nom;
#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate failure;

#[macro_use]
extern crate include_dir;

pub mod cdsl;
pub mod codegen;
pub mod nodes;
pub mod parse;
pub mod transform;

use crate::cdsl::*;
use failure::Error;
use include_dir::Dir;
use std::{
    borrow::Cow,
    fmt::Write,
    fs::{self, read_to_string, File},
    io::{stdin, Read},
    path::PathBuf,
    process::Command,
};
use structopt;
use structopt::StructOpt;
use tempdir::TempDir;

const RUNTIME_DIR: Dir<'_> = include_dir!("src/core");

#[derive(StructOpt, Debug)]
#[structopt(name = "somescheme")]
struct Opt {
    #[structopt(short = "d", long = "debug")]
    debug: bool,

    #[structopt(
        short = "o",
        long = "output",
        parse(from_os_str),
        default_value = "a.out"
    )]
    output: PathBuf,

    #[structopt(short = "i", long = "input", parse(from_os_str))]
    input: Option<PathBuf>,

    #[structopt(short = "k", long = "keep-tmp")]
    keep_tmpdir: bool,
}

fn main() -> Result<(), Error> {
    let opts = Opt::from_args();

    let input_exp = if let Some(input_path) = opts.input.as_ref() {
        read_to_string(input_path)?
    } else {
        let mut buf = String::new();
        stdin().read_to_string(&mut buf)?;
        buf
    };

    let expr = match parse::parse_exp(&input_exp) {
        Ok((_, expr)) => expr,
        Err(e) => {
            eprintln!("Error parsing input: {}", e);
            return Err(failure::err_msg("parse fail"));
        }
    };

    let (expr, ctx) = do_transforms(&opts, expr)?;

    let generated_source = do_codegen(&opts, expr, ctx)?;

    let full_source = generate_program_source(&generated_source);

    if opts.debug {
        eprintln!("{}", full_source);
    }

    let build_dir = generate_build_dir();

    insert_source_into_build_dir(&build_dir, &full_source);

    let make_stdout = match invoke_make(&build_dir) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            return Ok(());
        }
    };

    if opts.debug {
        eprintln!("{}", make_stdout);
    }

    copy_binary(&build_dir, &opts.output);

    if !opts.keep_tmpdir {
        build_dir.close()?;
    } else {
        println!("Temp dir: {}", build_dir.path().display());

        // don't close the temp dir by preventing it's drop
        std::mem::forget(build_dir);
    }

    Ok(())
}

fn copy_binary(tmp_dir: &TempDir, output_path: &PathBuf) {
    fs::copy(tmp_dir.path().join("compiled_result"), output_path)
        .expect("failed copying compiled binary");
}

fn invoke_make(tmp_dir: &TempDir) -> Result<String, Error> {
    let output = Command::new("make")
        .current_dir(tmp_dir.path())
        .output()
        .expect("Failed to build source");

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(format_err!(
            "Make failed with exit code: {:?}\nstdout: {}\n\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn insert_source_into_build_dir(tmp_dir: &TempDir, source: &str) {
    use std::io::Write;

    let tmp_path = tmp_dir.path().join("compiled_result.c");

    let mut file = File::create(tmp_path).unwrap();

    file.write_all(source.as_bytes())
        .expect("failed to write source into build dir file");
}

fn generate_build_dir() -> TempDir {
    use std::io::Write;

    let tmp_dir = TempDir::new("some_scheme").expect("unable to create temp dir");

    // copy in build files
    for file in RUNTIME_DIR.files() {
        let to_path = tmp_dir.path().join(file.path());

        let mut out_file = File::create(to_path).unwrap();

        out_file
            .write_all(file.contents())
            .expect("failed writing build file content into build dir");
    }

    tmp_dir
}

fn generate_program_source(src: &str) -> String {
    format!(
        "{}{}{}",
        r#"
#include <stdlib.h>
#include <string.h>
#include "base.h"
#include "builtin.h"
"#,
        src,
        r#"
int main() {
  struct env_table *base_env = alloca(ENV_TABLE_SIZE);
  memset(base_env, 0, ENV_TABLE_SIZE);
  base_env->base = object_base_new(OBJ_ENV);

  struct closure initial_closure = object_closure_one_new(0, main_lambda, base_env);
  struct thunk initial_thunk = {
    .closr = &initial_closure,
    .one = {NULL},
  };

  struct thunk *thnk_heap = malloc(sizeof(struct thunk));
  memcpy(thnk_heap, &initial_thunk, sizeof(struct thunk));
  scheme_start(thnk_heap);
}
"#
    )
}

fn do_transforms<'a>(
    opts: &Opt,
    r: nodes::LExpr<'a>,
) -> Result<(nodes::LExEnv<'a>, codegen::EnvCtx<'a>), Error> {
    let transforms = &[
        transform::rename_builtins,
        transform::transform_lits,
        transform::expand_lam_app,
        transform::expand_lam_body,
    ];

    if opts.debug {
        eprintln!("{:#?}", r);
    }

    let mut context = transform::TransformContext::default();

    if opts.debug {
        eprintln!("{}", r);
    }

    let r = transforms.into_iter().fold(r, |acc, func| {
        let r = func(acc, &mut context);

        if opts.debug {
            eprintln!("{0}\n{0:#?}", r);
        }

        r
    });

    let cont = nodes::LExpr::BuiltinIdent(Cow::from("halt_func"), nodes::LamType::OneArg);

    let r = transform::cps_transform_cont(r, cont, &mut context);

    if opts.debug {
        eprintln!("\n\ncps_transform: {0}\n\n{0:#?}", r);
    }

    let (expr, ctx) = codegen::resolve_env(r);

    Ok((expr, ctx))
}

fn do_codegen<'a>(
    opts: &Opt,
    expr: nodes::LExEnv<'a>,
    mut ctx: codegen::EnvCtx<'a>,
) -> Result<String, Error> {
    let mut output_buffer = String::new();

    if opts.debug {
        eprintln!("\n\nresolved_env: {0}\n\n{0:#?}", expr);
        eprintln!("{:#?}", ctx);
    }

    let (root, lambdas) = codegen::extract_lambdas(expr);
    if opts.debug {
        eprintln!("root: {:#?}\n\nlambdas: {:#?}", root, lambdas);
    }

    let lambdas_vec: Vec<_> = lambdas.values().cloned().collect();

    let compiled_lambdas = codegen::lambda_codegen(&lambdas_vec);

    if opts.debug {
        eprintln!("\nCompiled lambdas:\n");
        eprintln!("{:#?}", compiled_lambdas);
    }

    let lambda_protos = codegen::lambda_proto_codegen(&lambdas_vec);

    for lam_proto in &lambda_protos {
        writeln!(&mut output_buffer, "{}", lam_proto.export())?;
    }

    for lam in &compiled_lambdas {
        writeln!(&mut output_buffer, "{}", lam.export())?;
    }

    let mut supporting_stmts = Vec::new();
    let mut codegen_ctx = codegen::CodegenCtx::default();

    let compiled_root = codegen::codegen(&root, &mut codegen_ctx, &mut supporting_stmts);
    let compiled_root = cdsl::CStmt::Expr(compiled_root);

    supporting_stmts.push(compiled_root);

    let main_fn = cdsl::CDecl::Fun {
        name: Cow::from("main_lambda"),
        typ: cdsl::CType::Static(box cdsl::CType::Void),
        args: vec![
            (
                Cow::from("_"),
                CType::Ptr(box CType::Struct(Cow::from("object"))),
            ),
            (
                Cow::from("env"),
                CType::Ptr(box CType::Struct(Cow::from("env_table"))),
            ),
        ],
        body: supporting_stmts,
    };

    writeln!(&mut output_buffer, "{}", main_fn.export())?;

    let envs = ctx.lam_map.clone();
    let generated_env_ids = codegen::gen_env_ids(&mut ctx, &envs);

    for decl in &generated_env_ids {
        writeln!(&mut output_buffer, "{}", decl.export())?;
    }

    Ok(output_buffer)
}
