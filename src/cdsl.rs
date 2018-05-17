use std::{
    rc::Rc,
};

pub enum CExpr {
    BinOp {
        op: String,
        left: Rc<CExpr>,
        right: Rc<CExpr>,
    },
    PreUnOp {
        op: String,
        ex: Rc<CExpr>,
    },
    PostUnOp {
        op: String,
        ex: Rc<CExpr>,
    },
    ArrIndexOp {
        index: Rc<CExpr>,
        expr: Rc<CExpr>,
    },
    FunCallOp {
        expr: Rc<CExpr>,
        ands: Vec<Rc<CExpr>>,
    },
    Cast {
        ex: Rc<CExpr>,
        typ: Rc<CType>,
    },
    Lit(String),
}

pub enum CType {
    Ptr { to: Rc<CType>, },
    Arr { of: Rc<CType>, len: usize },
    Int { size: usize, sign: bool },
    Struct { name: String },
    Union { name: String },
}

pub enum CStmt {
    Fun {
        name: String,
        args: Vec<(String, Rc<CType>)>,
        body: Vec<Rc<CStmt>>,
    },
    IF {
        cond: Rc<CExpr>,
        body: Rc<CStmt>,
    },
    While {
        cond: Rc<CExpr>,
        body: Rc<CStmt>,
    },
    For {
        init: Rc<CExpr>,
        test: Rc<CExpr>,
        updt: Rc<CExpr>,
        body: Rc<CStmt>,
    },
    Block(Vec<Rc<CStmt>>),
    Expr(Rc<CExpr>),
}


pub trait ToC {
    fn export_internal(&self, s: &mut String);
    fn export(&self) -> String {
        let mut s = String::new();
        self.export_internal(&mut s);
        s
    }
}

macro_rules! export_helper {
    ($s:ident, str $e:expr) => ( $s.push_str($e) );
    ($s:ident, chr $e:expr) => ( $s.push($e) );
    ($s:ident, exp $e:ident) => ( $e.export_internal(&mut $s) );
    ($s:ident, $cmd:tt $op:tt $(, $innercmd:tt $innerop:tt)*) => {{
        export_helper!($s, $cmd $op);
        export_helper!($s, $($innercmd $innerop),*);
    }};
}

impl ToC for CExpr {
    fn export_internal(&self, mut s: &mut String) {
        use self::CExpr::*;

        match &self {
            BinOp {op, left, right} =>
                export_helper!(s, chr '(', exp left, chr ')', str op, chr '(', exp right, chr ')')
            ,
            PreUnOp {op, ex} => export_helper!(s, chr '(', exp ex, chr ')', str op),
            PostUnOp {op, ex} => export_helper!(s, str op, chr '(', exp ex, chr ')'),
            ArrIndexOp {index, expr} => export_helper!(s, chr '(', exp expr, str ")[", exp index, chr ')'),
            FunCallOp {expr, ands} => {
                export_helper!(s, chr '(', exp expr, str ")(");

                let len = ands.len();

                for expr in &ands[..len-1] {
                    expr.export_internal(&mut s);
                    s.push(',');
                }
                if let Some(last) = ands.last() {
                    last.export_internal(&mut s);
                }

                export_helper!(s, chr ')');
            },
            Cast {ex, typ} => panic!("not done yet"), // export_helper!(s, chr '(', exp typ , str ")(", exp ex, chr ')');
            Lit(lit) => export_helper!(s, chr '(', str lit, chr ')'),
        }
    }
}
