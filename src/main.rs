#![feature(box_syntax, box_patterns, or_patterns)]

pub mod base_expr;
pub mod cdsl;
pub mod codegen;
pub mod cont_expr;
pub mod expr;
pub mod flat_expr;
pub mod lifted_expr;
pub mod literals;
pub mod parse;
pub mod utils;

use base_expr::BExpr;
use cdsl::CDecl;
use cdsl::CExpr;
use cdsl::CStmt;
use cdsl::CType;
use cdsl::ToC;
use failure::{format_err, Error};
use include_dir::{include_dir, Dir};
use std::collections::HashMap;
use std::fmt::Write;
use std::rc::Rc;
use std::{
    fs::{self, read_to_string, File},
    io::{stdin, Read},
    path::PathBuf,
    process::Command,
};
use structopt::StructOpt;
use tempdir::TempDir;
use termcolor::ColorChoice;
use termcolor::StandardStream;

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

    let body = match parse::parse_body(&input_exp) {
        Ok((_, body)) => body,
        Err(e) => {
            eprintln!("Error parsing input: {}", e);
            return Err(failure::err_msg("parse fail"));
        }
    };

    let expr = BExpr::App(Rc::new(BExpr::Lam(Vec::new(), body)), Vec::new());

    if opts.debug {
        eprintln!("\n\nexpr after parsing: ");
        let _ = expr.pretty_print(StandardStream::stderr(ColorChoice::Auto));
        eprintln!("");
        eprintln!("\n\nexpr after binding: ");
        let _ = expr.clone().into_expr().pretty_print(StandardStream::stderr(ColorChoice::Auto));
        eprintln!("");
    }

    let k = Rc::new(cont_expr::KExpr::BuiltinIdent(moniker::Ignore(
        "exit".into(),
    )));

    if opts.debug {
        eprintln!("\n\nexpr after converting: ");
        let _ = expr
            .clone()
            .into_expr()
            .into_fexpr(k.clone())
            .pretty_print(StandardStream::stderr(ColorChoice::Auto));
        eprintln!("");
    }

    let (expr, lambdas) = expr.into_expr().into_fexpr(k).lift_lambdas();

    let generated_source = do_codegen(&opts, expr, lambdas)?;

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
  struct closure_obj initial_closure = object_closure_one_new(main_lambda, NULL);
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

fn do_codegen(
    opts: &Opt,
    expr: lifted_expr::LExpr,
    lambdas: HashMap<usize, lifted_expr::LiftedLambda>,
) -> Result<String, Error> {
    let mut output_buffer = String::new();

    if opts.debug {
        eprintln!("\n\nfinal expr before codegen: ");
        let _ = expr.pretty_print(StandardStream::stderr(ColorChoice::Auto));
        eprintln!("");

        for l in lambdas.values() {
            eprint!("lambda {}: ", l.id);
            let _ = l
                .body
                .pretty_print(StandardStream::stderr(ColorChoice::Auto));
            eprintln!("");
        }
    }

    let (mut root_stmts, protos, decls) = codegen::do_codegen(expr, &lambdas);

    for proto in &protos {
        writeln!(&mut output_buffer, "{}", proto.export())?;
    }

    for decl in &decls {
        writeln!(&mut output_buffer, "{}", decl.export())?;
    }

    root_stmts.push(Rc::new(CStmt::Expr(CExpr::MacroCall {
        name: "__builtin_unreachable".into(),
        args: vec![],
    })));

    let main_lambda = CDecl::Fun {
        name: "main_lambda".into(),
        typ: CType::Void,
        args: vec![("input_obj".into(), CType::Ptr(Rc::new(CType::Struct("obj".into())))),
                   ("input_env".into(), CType::Ptr(Rc::new(CType::Struct("env_obj".into()))))
        ],
        body: root_stmts,
    };

    writeln!(&mut output_buffer, "{}", main_lambda.export())?;

    Ok(output_buffer)
}
