#![feature(box_syntax, box_patterns)]

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
        typ: box CType::Ptr(box CType::Arr(box CType::Int { size: 8, sign: false},
                                           10)),
        args: vec![(Cow::Borrowed("a1"),
                    CType::Ptr(box CType::Int { size: 16, sign: false} ))],
        body: vec![CStmt::Expr(CExpr::Lit(Cow::Borrowed("lol")))],
    };

    println!("{}", fn_.export());

    let exp = "(lambda (x) (* x (fac (- x 1))))";
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

        let (r, ctx) = codegen::resolve_env(r);
        println!("\n\n{0}\n\n{0:#?}", r);
        println!("{:#?}", ctx);

        let (root, lambdas) = codegen::extract_lambdas(r);
        println!("{:#?}\n\n{:#?}", root, lambdas);

        let compiled_lambdas = codegen::lambda_codegen(&lambdas.values().map(|l| l.clone()).collect::<Vec<_>>());

        println!("{:#?}", compiled_lambdas);

        for lam in &compiled_lambdas {
            println!("{}", lam.export());
        }

        let compiled_root = codegen::codegen(&root);
        println!("{}", compiled_root.export());

    }
}
