#![feature(box_syntax, box_patterns)]

#[macro_use]
extern crate nom;

pub mod cdsl;
pub mod parse;
pub mod transform;

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

    if let Ok((_, r)) = parse::parse_exp("((lambda (x y) (lambda (lol) no) y) y)") {
        println!("{:?}", r);

        let resolved = transform::expand_lam(r);
        println!("{:?}", resolved);
    }
}
