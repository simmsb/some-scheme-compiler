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

    /// A lambda expanded to one parameter
    LamOne(Cow<'a, str>, Vec<LExpr<'a>>),

    /// A lambda with one parameter
    LamOneOne(Cow<'a, str>, Box<LExpr<'a>>),

    AppOne(Box<LExpr<'a>>, Box<LExpr<'a>>),

    AppOneCont(Box<LExpr<'a>>, Box<LExpr<'a>>, Cont<'a>),

    LamOneOneCont(Cow<'a, str>, Box<LExpr<'a>>, Cont<'a>),
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
            LamOneOne(arg, box expr) =>
                write!(f, "(lambda ({}) {})", arg, expr),
            AppOne(box operator, box operands) =>
                write!(f, "({} {})", operator, operands),
            LamOne(arg, body) => {
                write!(f, "(lambda ({})", arg)?;
                for expr in body {
                    write!(f, " {}", expr)?;
                }
                write!(f, ")")
            },
            LamOneOneCont(arg, box expr, box cont) =>
                write!(f, "(lambda ({} {}) {})", arg, cont, expr),
            AppOneCont(box operator, box operand, box cont) =>
                write!(f, "({} {} {})", operator, operand, cont),
        }
    }
}

