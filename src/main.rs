#![feature(box_syntax, box_patterns)]


#[macro_use]
extern crate nom;
#[macro_use]
extern crate derive_more;
extern crate itertools;

pub mod cdsl;
pub mod parse;
pub mod transform;
pub mod nodes;
pub mod codegen;

use cdsl::*;
use std::borrow::Cow;

fn main() {
    let transforms = &[
        transform::rename_builtins,
        transform::transform_lits,
        transform::expand_lam_app,
        transform::expand_lam_body,
    ];

    let exp = "(+ 1 2)";
    if let Ok((_, mut r)) = parse::parse_exp(exp) {
        eprintln!("{:#?}", r);

        let mut context = transform::TransformContext::default();

        eprintln!("{}", r);

        for transform in transforms {
            r = transform(r, &mut context);
            eprintln!("{0}\n{0:#?}", r);
        }

        let cont = nodes::LExpr::BuiltinIdent(Cow::from("halt_func"));

        let r = transform::cps_transform_cont(r, cont, &mut context);
        eprintln!("\n\ncps_transform: {0}\n\n{0:#?}", r);


        let (r, mut ctx) = codegen::resolve_env(r);
        eprintln!("\n\nresolved_env: {0}\n\n{0:#?}", r);
        eprintln!("{:#?}", ctx);

        let (root, lambdas) = codegen::extract_lambdas(r);
        eprintln!("root: {:#?}\n\nlambdas: {:#?}", root, lambdas);

        let lambdas_vec: Vec<_> = lambdas.values().cloned().collect();

        let compiled_lambdas = codegen::lambda_codegen(&lambdas_vec);

        eprintln!("\nCompiled lambdas:\n");
        eprintln!("{:#?}", compiled_lambdas);

        let lambda_protos = codegen::lambda_proto_codegen(&lambdas_vec);

        for lam_proto in &lambda_protos {
            println!("{}", lam_proto.export());
        }

        for lam in &compiled_lambdas {
            println!("{}", lam.export());
        }


        let mut supporting_stmts = Vec::new();
        let mut codegen_ctx = codegen::CodegenCtx::default();

        let compiled_root = codegen::codegen(&root, &mut codegen_ctx, &mut supporting_stmts);
        let compiled_root = cdsl::CStmt::Expr(compiled_root);

        supporting_stmts.push(compiled_root);

        // eprintln!("supporting: {:#?}", supporting_stmts);

        let main_fn = cdsl::CDecl::Fun {
            name: Cow::from("main_lambda"),
            typ: cdsl::CType::Void,
            args: vec![
                (Cow::from("_"), CType::Ptr(box CType::Struct(Cow::from("object")))),
                (Cow::from("env"), CType::Ptr(box CType::Struct(Cow::from("env_elem"))))],
            body: supporting_stmts,
        };
        // eprintln!("\nFinal result:\n");
        println!("{}", main_fn.export());

        let envs = ctx.lam_map.clone();
        let generated_env_ids = codegen::gen_env_ids(&mut ctx, &envs);

        for decl in &generated_env_ids {
            println!("{}", decl.export());
        }
    }
}
