use itertools::Itertools;
use std::{
    boxed::Box,
    borrow::Cow,
    fmt,
    rc::Rc,
    collections::HashSet,
};

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


#[derive(Debug, Clone)]
pub struct Env<'a> {
    this: HashSet<Cow<'a, str>>,
    parent: Option<Rc<Env<'a>>>,
}


impl<'a> Env<'a> {
    fn new(parent: Option<Rc<Env<'a>>>) -> Self {
        Env {
            this: HashSet::new(),
            parent
        }
    }
}


/// Expressions that have an explicit environment.
#[derive(Debug, Clone)]
pub enum LExEnv<'a> {
    Lam { arg: Cow<'a, str>,
          expr: Box<LExEnv<'a>>,
          cont: Box<LExEnv<'a>>,
          env: Rc<Env<'a>>
    },
    App { rator: Box<LExEnv<'a>>,
          rand: Box<LExEnv<'a>>,
          cont: Box<LExEnv<'a>>,
          env: Rc<Env<'a>>
    },
    Var { name: Cow<'a, str>,
          global: bool,
          env: Rc<Env<'a>>
    },
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

