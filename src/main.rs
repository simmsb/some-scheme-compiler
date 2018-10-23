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
    let fn_ = CDecl::Fun {
        name: Cow::from("lol"),
        typ: CType::Ptr(box CType::Arr(box CType::Int { size: 8, sign: false},
                                       Some(10))),
        args: vec![(Cow::from("a1"),
                    CType::Ptr(box CType::Int { size: 16, sign: false} ))],
        body: vec![CStmt::Expr(CExpr::LitStr(Cow::from("lol")))],
    };

    println!("{}", fn_.export());

    let transforms = &[
        transform::rename_builtins,
        transform::transform_lits,
        transform::expand_lam_app,
        transform::expand_lam_body,
    ];

    let exp = "(+ 1 2)";
    if let Ok((_, mut r)) = parse::parse_exp(exp) {
        println!("{:#?}", r);

        let mut context = transform::TransformContext::default();

        println!("{}", r);

        for transform in transforms {
            r = transform(r, &mut context);
            println!("{0}\n{0:#?}", r);
        }

        let cont = nodes::LExpr::BuiltinIdent(Cow::from("halt_func"));

        // There's a bug here that's causing some stuff to be discarded
        // not sure where exactly

        let r = transform::cps_transform_cont(r, cont, &mut context);
        println!("\n\ncps_transform: {0}\n\n{0:#?}", r);


        let (r, mut ctx) = codegen::resolve_env(r);
        println!("\n\nresolved_env: {0}\n\n{0:#?}", r);
        println!("{:#?}", ctx);

        let (root, lambdas) = codegen::extract_lambdas(r);
        println!("root: {:#?}\n\nlambdas: {:#?}", root, lambdas);

        let lambdas_vec: Vec<_> = lambdas.values().cloned().collect();

        let compiled_lambdas = codegen::lambda_codegen(&lambdas_vec);

        println!("\nCompiled lambdas:\n");
        println!("{:#?}", compiled_lambdas);

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

        // println!("supporting: {:#?}", supporting_stmts);

        let main_fn = cdsl::CDecl::Fun {
            name: Cow::from("main_lambda"),
            typ: cdsl::CType::Void,
            args: vec![(Cow::from("env"), CType::Ptr(box CType::Struct(Cow::from("env_elem"))))],
            body: supporting_stmts,
        };
        // println!("\nFinal result:\n");
        println!("{}", main_fn.export());

        let envs = ctx.lam_map.clone();
        let generated_env_ids = codegen::gen_env_ids(&mut ctx, &envs);

        for decl in &generated_env_ids {
            println!("{}", decl.export());
        }
    }
}
