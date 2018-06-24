#![feature(box_syntax, box_patterns, iterator_flatten)]

#[macro_use]
extern crate nom;
extern crate itertools;

pub mod cdsl;
pub mod parse;
pub mod transform;
pub mod nodes;
pub mod codegen;

use cdsl::*;
use std::borrow::Cow;

fn main() {
    let fn_ = CDecl::Fun {
        name: Cow::Borrowed("lol"),
        typ: CType::Ptr(box CType::Arr(box CType::Int { size: 8, sign: false},
                                       Some(10))),
        args: vec![(Cow::Borrowed("a1"),
                    CType::Ptr(box CType::Int { size: 16, sign: false} ))],
        body: vec![CStmt::Expr(CExpr::LitStr(Cow::Borrowed("lol")))],
    };

    println!("{}", fn_.export());

    let exp = "(+ 1 2)";
    if let nom::IResult::Done(_, r) = parse::parse_exp(exp) {
        println!("{:#?}", r);

        let mut context = transform::TransformContext::new();

        println!("{}", r);
        let r = transform::expand_lam_app(r, &mut context);
        println!("{0}\n{0:#?}", r);
        let r = transform::expand_lam_body(r, &mut context);
        println!("{0}\n{0:#?}", r);

        let (_, cont) = parse::parse_var("halt").unwrap();

        let r = transform::cps_transform_cont(r, cont, &mut context);
        println!("\n\n{0}\n\n{0:#?}", r);


        let (r, mut ctx) = codegen::resolve_env(r);
        println!("\n\n{0}\n\n{0:#?}", r);
        println!("{:#?}", ctx);

        let (root, lambdas) = codegen::extract_lambdas(r);
        println!("{:#?}\n\n{:#?}", root, lambdas);

        let compiled_lambdas = codegen::lambda_codegen(&lambdas.values().map(|l| l.clone()).collect::<Vec<_>>());

        println!("{:#?}", compiled_lambdas);

        for lam in &compiled_lambdas {
            println!("{}", lam.export());
        }


        let mut supporting_stmts = Vec::new();
        let mut codegen_ctx = codegen::CodegenCtx::new();

        let compiled_root = codegen::codegen(&root, &mut codegen_ctx, &mut supporting_stmts);

        let compiled_root = cdsl::CStmt::Expr(compiled_root);

        supporting_stmts.push(compiled_root);

        let main_fn = cdsl::CDecl::Fun {
            name: Cow::Borrowed("main"),
            typ: cdsl::CType::Void,
            args: vec![],
            body: supporting_stmts,
        };

        println!("{}", main_fn.export());

        let envs = ctx.lam_map.clone();
        let generated_env_ids = codegen::gen_env_ids(&mut ctx, envs);

        for decl in &generated_env_ids {
            println!("{}", decl.export());
        }
    }
}
