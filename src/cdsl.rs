use std::fmt::Debug;
use std::{borrow::Cow, rc::Rc};
use std::fmt::Write;

pub trait ToCDC: ToC + Debug {}
impl<T: ToC + Debug> ToCDC for T {}

#[derive(Debug)]
pub enum CExpr<'a> {
    BinOp {
        op: Cow<'a, str>,
        left: Rc<CExpr<'a>>,
        right: Rc<CExpr<'a>>,
    },
    PreUnOp {
        op: Cow<'a, str>,
        ex: Rc<CExpr<'a>>,
    },
    PostUnOp {
        op: Cow<'a, str>,
        ex: Rc<CExpr<'a>>,
    },
    ArrIndexOp {
        index: Rc<CExpr<'a>>,
        expr: Rc<CExpr<'a>>,
    },
    Dot {
        expr: Rc<CExpr<'a>>,
        attr: Cow<'a, str>,
    },
    Arrow {
        expr: Rc<CExpr<'a>>,
        attr: Cow<'a, str>,
    },
    FunCallOp {
        expr: Rc<CExpr<'a>>,
        params: Vec<Rc<CExpr<'a>>>,
    },
    Cast {
        ex: Rc<CExpr<'a>>,
        typ: CType<'a>,
    },
    MacroCall {
        name: Cow<'a, str>,
        args: Vec<Rc<dyn ToCDC + 'a>>,
    },
    If {
        cond: Rc<CExpr<'a>>,
        ift: Rc<CExpr<'a>>,
        iff: Rc<CExpr<'a>>,
    },
    InitList(Vec<CExpr<'a>>),
    Ident(Cow<'a, str>),
    LitStr(Cow<'a, str>),
    LitUInt(usize),
    LitIInt(isize),
}

#[derive(Debug)]
pub enum CType<'a> {
    Ptr(Rc<CType<'a>>),
    Arr(Rc<CType<'a>>, Option<usize>),
    Int { size: usize, sign: bool },
    Struct(Cow<'a, str>),
    Union(Cow<'a, str>),
    Other(Cow<'a, str>),
    Static(Rc<CType<'a>>),
    Const(Rc<CType<'a>>),
    Void,
}

#[derive(Debug)]
pub enum CStmt<'a> {
    If {
        cond: CExpr<'a>,
        ift: Rc<CStmt<'a>>,
        iff: Rc<CStmt<'a>>,
    },
    While {
        cond: CExpr<'a>,
        body: Rc<CStmt<'a>>,
    },
    For {
        init: CExpr<'a>,
        test: CExpr<'a>,
        updt: CExpr<'a>,
        body: Rc<CStmt<'a>>,
    },
    Decl(CDecl<'a>),
    Block(Vec<Rc<CStmt<'a>>>),
    Expr(CExpr<'a>),
}

#[derive(Debug)]
pub enum CDecl<'a> {
    FunProto {
        name: Cow<'a, str>,
        typ: CType<'a>,
        args: Vec<CType<'a>>,
        noreturn: bool,
    },
    Fun {
        name: Cow<'a, str>,
        typ: CType<'a>,
        args: Vec<(Cow<'a, str>, CType<'a>)>,
        body: Vec<Rc<CStmt<'a>>>,
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
        typ: CType<'a>,
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
    ($s:ident, vec_csep $e:expr) => ({
        let mut it = $e.into_iter();
        if let Some(expr) = it.next() {
            expr.export_internal(&mut $s);
        }
        for elem in it {
            let _ = write!($s, ",");
            elem.export_internal(&mut $s);
        }
    });
    ($s:ident, $cmd:tt $op:tt $(, $innercmd:tt $innerop:tt)*) => {{
        export_helper!($s, $cmd $op);
        export_helper!($s, $($innercmd $innerop),*);
    }};
}

impl<'a> ToC for CExpr<'a> {
    fn export_internal(&self, mut s: &mut String) {
        use self::CExpr::*;

        match self {
            BinOp { op, left, right } => export_helper!(
                s,
                chr '(',
                exp left,
                chr ')',
                str op,
                chr '(',
                exp right,
                chr ')'
            ),
            PreUnOp { op, ex } => export_helper!(
                s,
                str op,
                chr '(',
                exp ex,
                chr ')'
            ),
            PostUnOp { op, ex } => export_helper!(
                s,
                chr '(',
                exp ex,
                chr ')',
                str op
            ),
            ArrIndexOp { index, expr } => export_helper!(
                s,
                chr '(',
                exp expr,
                str ")[",
                exp index,
                chr ')'
            ),
            Dot { expr, attr } => export_helper!(
                s, chr '(', exp expr, str ").", str attr
            ),
            Arrow { expr, attr } => export_helper!(
                s, chr '(', exp expr, str ")->", str attr
            ),
            FunCallOp { expr, params } => export_helper!(
                s,
                chr '(',
                exp expr,
                str ")(",
                vec_csep params,
                chr ')'
            ),
            Cast { ex, typ } => export_helper!(
                s,
                chr '(',
                exp typ,
                str ")(",
                exp ex,
                chr ')'
            ),
            MacroCall { name, args } => export_helper!(
                s,
                str name,
                chr '(',
                vec_csep args,
                chr ')'
            ),
            InitList(exprs) => export_helper!(
                s,
                chr '{',
                vec_csep exprs,
                chr '}'
            ),
            If { cond, ift, iff } => export_helper!(
                s,
                chr '(',
                exp cond,
                str ")?(",
                exp ift,
                str "):(",
                exp iff,
                chr ')'
            ),
            Ident(name) => s.push_str(name),
            LitStr(lit) => export_helper!(
                s,
                chr '"',
                str lit,
                chr '"'
            ),
            LitUInt(lit) => export_helper!(s, str &lit.to_string()),
            LitIInt(lit) => export_helper!(s, str &lit.to_string()),

        }
    }
}

impl<'a> ToC for CType<'a> {
    fn export_internal(&self, s: &mut String) {
        self.export_with_name(s, &|_s| {});
    }
}

impl<'a> CType<'a> {
    fn export_with_name(&self, s: &mut String, name_writer: &dyn Fn(&mut String)) {
        use self::CType::*;

        match self {
            Ptr(to) =>
                to.export_with_name(s, &|s| {
                    let _ = write!(s, "*");
                    name_writer(s);
                    let _ = write!(s, "");
                }),
            Arr(of, None) =>
                of.export_with_name(s, &|s| {
                    let _ = write!(s, "");
                    name_writer(s);
                    s.push_str("[]");
                }),
            Arr(of, Some(len)) =>
                of.export_with_name(s, &|s| {
                    let _ = write!(s, "");
                    name_writer(s);
                    let _ = write!(s, "[{}]", len);
                }),
            Int { size, sign } => {
                let _ = write!(s, "{}int{}_t", if *sign { "u" } else { "" }, size);
                name_writer(s);
            }
            Struct(tname) => {
                let _ = write!(s, "struct {} ", tname);
                name_writer(s);
            }
            Union(tname) => {
                let _ = write!(s, "union {} ", tname);
                name_writer(s);
            }
            Other(tname) => {
                let _ = write!(s, "{} ", tname);
                name_writer(s);
            }
            Static(of) =>
                of.export_with_name(s, &|s| {
                    let _ = write!(s, "static ");
                    name_writer(s);
                }),
            Const(of) =>
                of.export_with_name(s, &|s| {
                    let _ = write!(s, "const ");
                    name_writer(s);
                }),
            Void => {
                let _ = write!(s, "void ");
                name_writer(s);
            }
        };
    }
}

impl<'a> ToC for CStmt<'a> {
    fn export_internal(&self, mut s: &mut String) {
        use self::CStmt::*;

        match self {
            If { cond, ift, iff } => export_helper!(
                s,
                str "if (",
                exp cond,
                chr ')',
                exp ift,
                str " else ",
                exp iff
            ),
            While { cond, body } => export_helper!(
                s,
                str "while (",
                exp cond,
                chr ')',
                exp body
            ),
            For {
                init,
                test,
                updt,
                body,
            } => export_helper!(
                s,
                str "for (",
                exp init,
                chr ';',
                exp test,
                chr ';',
                exp updt,
                chr ')',
                exp body
            ),
            Decl(decl) => export_helper!(s, exp decl),
            Block(body) => export_helper!(
                s,
                chr '{',
                vec body,
                chr '}'
            ),
            Expr(body) => export_helper!(
                s,
                exp body,
                chr ';'
            ),
        }
    }
}

impl<'a> ToC for CDecl<'a> {
    fn export_internal(&self, mut s: &mut String) {
        use self::CDecl::*;

        match self {
            FunProto {
                name,
                typ,
                args,
                noreturn,
            } => {
                typ.export_with_name(s, &|s| {
                    let _ = write!(s, "{}(", name);

                    let mut it = args.iter();

                    if let Some(atyp) = it.next() {
                        atyp.export_with_name(s, &|_s| {});
                    }

                    for atyp in it {
                        let _ = write!(s, ", ");
                        atyp.export_with_name(s, &|_s| {});
                    }

                    let _ = write!(s, ")");

                    if *noreturn {
                        let _ = write!(s, "__attribute__((noreturn)) ");
                    }
                });

                let _ = write!(s, ";");
            }
            Fun {
                name,
                typ,
                args,
                body,
            } => {
                typ.export_with_name(s, &|s| {
                    let _ = write!(s, "{}(", name);

                    let mut it = args.iter();

                    if let Some((aname, atyp)) = it.next() {
                        atyp.export_with_name(s, &|s| { s.push_str(aname); });
                    }

                    for (aname, atyp) in it {
                        let _ = write!(s, ", ");
                        atyp.export_with_name(s, &|s| { s.push_str(aname); });
                    }

                    let _ = write!(s, ")");
                });

                export_helper!(s,
                               chr '{',
                               vec body,
                               chr '}'
                );
            }
            Struct { name, members } => {
                let _ = write!(s, "struct {} {{", name);

                for (aname, atyp) in members {
                    atyp.export_with_name(s, &|s| { s.push_str(aname); });
                    let _ = write!(s, ";");
                }
                let _ = write!(s, "}};");
            }
            Union { name, members } => {
                let _ = write!(s, "union {} {{", name);

                for (aname, atyp) in members {
                    atyp.export_with_name(s, &|s| { s.push_str(aname); });
                    let _ = write!(s, ";");
                }
                let _ = write!(s, "}};");
            }
            Var { name, typ, init } => {
                typ.export_with_name(s, &|s| { s.push_str(name); });
                if let Some(init) = init {
                    export_helper!(s, str " = ", exp init);
                }
                let _ = write!(s, ";");
            }
        }
    }
}
