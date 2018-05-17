pub mod cdsl;

use std::rc::Rc;
use cdsl::*;

fn main() {
    let op = CExpr::BinOp {
        op: "+".to_owned(),
        left: Rc::new(CExpr::Lit("wew".to_owned())),
        right: Rc::new(CExpr::Lit("lad".to_owned())),
    };

    println!("{}", op.export());
}
