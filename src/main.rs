#![feature(box_syntax, box_patterns)]

#[macro_use]
extern crate nom;
extern crate itertools;

pub mod cdsl;
pub mod parse;
pub mod transform;
pub mod nodes;

use cdsl::*;

fn main() {
    let fn_ = CDecl::Fun {
        name: "lol".to_owned(),
        typ: box CType::Ptr { to: box CType::Arr { of: box CType::Int { size: 8, sign: false},
                                                   len: 10}},
        args: vec![("a1".to_owned(), box CType::Ptr { to: box CType::Int { size: 16, sign: false} })],
        body: vec![box CStmt::Expr( box CExpr::Lit("lol".to_owned()))],
    };

    println!("{}", fn_.export());

    if let Ok((_, r)) = parse::parse_exp("((lambda () (y) (x)))") {
        println!("{:?}", r);

        let mut context = transform::TransformContext::new();

        println!("{}", r);
        let r = transform::expand_lam_app(r, &mut context);
        println!("{0}\n{0:?}", r);
        let r = transform::expand_lam_body(r, &mut context);
        println!("{0}\n{0:?}", r);

        let (_, cont) = parse::parse_exp("(halt)").unwrap();

        let r = transform::cps_transform_cont(r, cont, &mut context);
        println!("\n\n{0}\n\n{0:?}", r);
    }
}
