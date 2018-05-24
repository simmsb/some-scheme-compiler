use std::boxed::Box;
use itertools::Itertools;
use std::borrow::Cow;
use std::fmt;

type Cont<'a> = Box<LExpr<'a>>;

#[derive(Debug, Clone)]
pub enum LExpr<'a> {
    Lam(Vec<Cow<'a, str>>, Vec<LExpr<'a>>),
    App(Box<LExpr<'a>>, Vec<LExpr<'a>>),
    Var(Cow<'a, str>),

    /// A lambda with no parameters
    LamNone(Vec<LExpr<'a>>),

    /// A lambda expanded to one parameter
    LamOne(Cow<'a, str>, Vec<LExpr<'a>>),

    /// A lambda with no parameters
    LamNoneOne(Box<LExpr<'a>>),
    LamNoneNone, // for (lambda ())

    /// A lambda with one parameter
    LamOneOne(Cow<'a, str>, Box<LExpr<'a>>),
    LamOneNone(Cow<'a, str>), // for (lambda (_))

    /// An application with zero arguments
    AppNone(Box<LExpr<'a>>),

    /// An application expanded to only one argument
    AppOne(Box<LExpr<'a>>, Box<LExpr<'a>>),

    AppOneCont(Box<LExpr<'a>>, Box<LExpr<'a>>, Cont<'a>),

    LamNoneOneCont(Box<LExpr<'a>>, Cont<'a>),
    LamNoneNoneCont(Cont<'a>),
    LamOneOneCont(Cow<'a, str>, Box<LExpr<'a>>, Cont<'a>),
    LamOneNoneCont(Cow<'a, str>, Cont<'a>),
}


impl<'a> fmt::Display for LExpr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use nodes::LExpr::*;

        match self {
            Lam(args, body) => {
                write!(f, "(lambda ({})", args.iter().join(" "))?;
                for expr in body {
                    write!(f, " {}", expr)?;
                }
                write!(f, ")")
            },
            App(box operator, operands) => {
                write!(f, "({}", operator)?;
                for operand in operands {
                    write!(f, " {}", operand)?;
                }
                write!(f, ")")
            },
            Var(name) =>
                write!(f, "{}", name),
            LamNoneOne(box expr) =>
                write!(f, "(lambda () {})", expr),
            LamNoneNone =>
                write!(f, "(lambda ())"),
            LamOneOne(arg, box expr) =>
                write!(f, "(lambda ({}) {})", arg, expr),
            LamOneNone(arg) =>
                write!(f, "(lambda ({}))", arg),
            AppNone(box operator) =>
                write!(f, "({})", operator),
            AppOne(box operator, box operands) =>
                write!(f, "({} {})", operator, operands),
            LamNone(body) => {
                write!(f, "(lambda ()")?;
                for expr in body {
                    write!(f, " {}", expr)?;
                }
                write!(f, ")")
            },
            LamOne(arg, body) => {
                write!(f, "(lambda ({})", arg)?;
                for expr in body {
                    write!(f, " {}", expr)?;
                }
                write!(f, ")")
            },
            LamNoneOneCont(box expr, box cont) =>
                write!(f, "(lambda ({}) {})", cont, expr),
            LamNoneNoneCont(box cont) =>
                write!(f, "(lambda ({}))", cont),
            LamOneOneCont(arg, box expr, box cont) =>
                write!(f, "(lambda ({} {}) {})", arg, cont, expr),
            LamOneNoneCont(arg, box cont) =>
                write!(f, "(lambda ({} {}))", arg, cont),
            AppOneCont(box operator, box operand, box cont) =>
                write!(f, "({} {} {})", operator, operand, cont),
        }
    }
}

