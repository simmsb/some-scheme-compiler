pub mod cdsl;

use std::rc::Rc;
use cdsl::*;

fn main() {
    let fn_ = CDecl::Fun {
        name: "lol".to_owned(),
        typ: Rc::new(CType::Ptr { to: Rc::new(CType::Arr { of: Rc::new(CType::Int { size: 8, sign: false}),
                                                           len: 10})}),
        args: vec![("a1".to_owned(), Rc::new(CType::Ptr { to: Rc::new( CType::Int { size: 16, sign: false} )}))],
        body: vec![Rc::new(CStmt::Expr(Rc::new(CExpr::Lit("lol".to_owned()))))],
    };

    println!("{}", fn_.export());
}
