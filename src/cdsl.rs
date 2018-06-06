use std::{
    boxed::Box,
    borrow::Cow,
};

pub enum CExpr<'a> {
    BinOp {
        op: Cow<'a, str>,
        left: Box<CExpr<'a>>,
        right: Box<CExpr<'a>>,
    },
    PreUnOp {
        op: Cow<'a, str>,
        ex: Box<CExpr<'a>>,
    },
    PostUnOp {
        op: Cow<'a, str>,
        ex: Box<CExpr<'a>>,
    },
    ArrIndexOp {
        index: Box<CExpr<'a>>,
        expr: Box<CExpr<'a>>,
    },
    FunCallOp {
        expr: Box<CExpr<'a>>,
        ands: Vec<CExpr<'a>>,
    },
    Cast {
        ex: Box<CExpr<'a>>,
        typ: Box<CType<'a>>,
    },
    Lit(Cow<'a, str>),
}

pub enum CType<'a> {
    Ptr(Box<CType<'a>>),
    Arr(Box<CType<'a>>, usize),
    Int { size: usize, sign: bool },
    Struct(Cow<'a, str>),
    Union(Cow<'a, str>),
    Void,
}

pub enum CStmt<'a> {
    IF {
        cond: CExpr<'a>,
        body: Box<CStmt<'a>>,
    },
    While {
        cond: CExpr<'a>,
        body: Box<CStmt<'a>>,
    },
    For {
        init: CExpr<'a>,
        test: CExpr<'a>,
        updt: CExpr<'a>,
        body: Box<CStmt<'a>>,
    },
    Block(Vec<CStmt<'a>>),
    Expr(CExpr<'a>),
}

pub enum CDecl<'a> {
    Fun {
        name: Cow<'a, str>,
        typ: Box<CType<'a>>,
        args: Vec<(Cow<'a, str>, CType<'a>)>,
        body: Vec<CStmt<'a>>,
    },
    Struct {
        name: Cow<'a, str>,
        members: Vec<(Cow<'a, str>, CType<'a>)>,
    },
    Union {
        name: Cow<'a, str>,
        members: Vec<(Cow<'a, str>, CType<'a>)>,
    },
    Var {
        name: Cow<'a, str>,
        typ: Box<CType<'a>>,
        init: Option<CExpr<'a>>,
    },
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
    ($s:ident, vec $e:expr) => ( for elem in $e { elem.export_internal(&mut $s); } );
    ($s:ident, $cmd:tt $op:tt $(, $innercmd:tt $innerop:tt)*) => {{
        export_helper!($s, $cmd $op);
        export_helper!($s, $($innercmd $innerop),*);
    }};
}

impl<'a> ToC for CExpr<'a> {
    fn export_internal(&self, mut s: &mut String) {
        use self::CExpr::*;

        match self {
            BinOp { op, left, right } => {
                export_helper!(s, chr '(', exp left, chr ')', str op, chr '(', exp right, chr ')')
            }
            PreUnOp { op, ex } => export_helper!(s, chr '(', exp ex, chr ')', str op),
            PostUnOp { op, ex } => export_helper!(s, str op, chr '(', exp ex, chr ')'),
            ArrIndexOp { index, expr } => {
                export_helper!(s, chr '(', exp expr, str ")[", exp index, chr ')')
            }
            FunCallOp { expr, ands } => {
                export_helper!(s, chr '(', exp expr, str ")(");

                let mut it = ands.iter();

                if let Some(expr) = it.next() {
                    expr.export_internal(&mut s);
                }

                for expr in it {
                    s.push(',');
                    expr.export_internal(&mut s);
                }

                export_helper!(s, chr ')');
            }
            Cast { ex, typ } => export_helper!(s, chr '(', exp typ , str ")(", exp ex, chr ')'),
            Lit(lit) => export_helper!(s, chr '(', str lit, chr ')'),
        }
    }
}

impl<'a> ToC for CType<'a> {
    fn export_internal(&self, s: &mut String) {
        s.push_str(&self.export_with_name(""));
    }
}

impl<'a> CType<'a> {
    fn export_with_name(&self, name: &str) -> String {
        use self::CType::*;

        let mut typ = Some(self);
        let mut gen = name.to_owned();

        while let Some(typ_o) = typ.take() {
            gen = match typ_o {
                Ptr(..) => format!("*{}", gen),
                Arr(_, len) => format!("({})[{}]", gen, len),
                Int {size, sign} => {
                    let name = format!("{}int{}_t",
                                       if *sign { "u" } else { "" },
                                       size);
                    format!("{} {}", name, gen)
                },
                Struct(name) => format!("struct {} {}", name, gen),
                Union(name) => format!("union {} {}", name, gen),
                Void => "void".to_owned(),
            };

            match typ_o {
                Ptr(to) => typ = Some(to),
                Arr(of, ..) => typ = Some(of),
                _ => (),
            };
        }

        return gen;
    }
}

impl<'a> ToC for CStmt<'a> {
    fn export_internal(&self, mut s: &mut String) {
        use self::CStmt::*;

        match self {
            IF {cond, body} => export_helper!(s, str "if (", exp cond, chr ')', exp body),
            While {cond, body} => export_helper!(s, str "while (", exp cond, chr ')', exp body),
            For {init, test, updt, body} => export_helper!(s, str "for (", exp init, chr ';', exp test, chr ';', exp updt, chr ')', exp body),
            Block(body) => export_helper!(s, chr '{', vec body, chr '}'),
            Expr(body) => export_helper!(s, exp body, chr ';'),
        }
    }
}

impl<'a> ToC for CDecl<'a> {
    fn export_internal(&self, mut s: &mut String) {
        use self::CDecl::*;

        match self {
            Fun {name, typ, args, body} => {
                let mut f = String::new();

                f.push_str(&name);
                f.push('(');

                let mut it = args.iter();

                if let Some((aname, atyp)) = it.next() {
                    f.push_str(&atyp.export_with_name(aname));
                }

                for (aname, atyp) in it {
                    s.push(',');
                    f.push_str(&atyp.export_with_name(aname));
                }

                f.push(')');

                s.push_str(&typ.export_with_name(&f));

                export_helper!(s, chr '{', vec body, chr '}');
            },
            Struct {name, members} => {
                export_helper!(s, str "struct ", str name, chr '{');

                for (aname, atyp) in members {
                    s.push_str(&atyp.export_with_name(aname));
                    s.push(';');
                }
                s.push(';');
            },
            Union {name, members} => {
                export_helper!(s, str "union ", str name, chr '{');

                for (aname, atyp) in members {
                    s.push_str(&atyp.export_with_name(aname));
                    s.push(';');
                }
                s.push(';');
            },
            Var {name, typ, init} => {
                s.push_str(&typ.export_with_name(name));
                if let Some(init) = init {
                    export_helper!(s, exp init);
                }
                s.push(';');
            }

        }
    }
}
