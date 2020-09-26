use moniker::BoundTerm;
use moniker::FreeVar;
use moniker::{Ignore, Var};

use pretty::{BoxAllocator, DocAllocator, DocBuilder};
use termcolor::{Color, ColorSpec, WriteColor};

use std::collections::HashSet;
use std::{io::Result, rc::Rc};

use crate::literals::Literal;

#[derive(Debug, Clone, BoundTerm)]
pub enum LExpr {
    Var(Var<String>),
    Lit(Ignore<Literal>),
    BuiltinIdent(Ignore<String>),
    SetThen(Var<String>, Rc<LExpr>, Rc<LExpr>),
    If(Rc<LExpr>, Rc<LExpr>, Rc<LExpr>),
    Lifted(Ignore<usize>),
    CallOne(Rc<LExpr>, Rc<LExpr>),
    CallTwo(Rc<LExpr>, Rc<LExpr>, Rc<LExpr>),
}

#[derive(Debug, Clone)]
pub struct LiftedLambda {
    pub id: usize,
    pub params: Vec<FreeVar<String>>,
    pub freevars: HashSet<FreeVar<String>>,
    pub body: Rc<LExpr>,
}

impl LiftedLambda {
    pub fn new(
        id: usize,
        params: Vec<FreeVar<String>>,
        freevars: HashSet<FreeVar<String>>,
        body: Rc<LExpr>,
    ) -> Self {
        Self {
            id,
            params,
            freevars,
            body,
        }
    }
}

impl LExpr {
    pub fn pretty<'a, D>(&self, allocator: &'a D) -> DocBuilder<'a, D, ColorSpec>
    where
        D: DocAllocator<'a, ColorSpec>,
        D::Doc: Clone,
    {
        match self {
            LExpr::Var(s) => allocator.as_string(s),
            LExpr::Lit(Ignore(l)) => l.pretty(allocator),
            LExpr::BuiltinIdent(Ignore(s)) => allocator.as_string(s),
            LExpr::SetThen(n, v, c) => {
                let v_pret = v.pretty(allocator);
                let c_pret = c.pretty(allocator);

                allocator
                    .text("set-then!")
                    .annotate(ColorSpec::new().set_fg(Some(Color::Magenta)).clone())
                    .append(allocator.space())
                    .append(
                        allocator
                            .as_string(n)
                            .annotate(ColorSpec::new().set_fg(Some(Color::Green)).clone()),
                    )
                    .append(allocator.space())
                    .append(v_pret)
                    .append(allocator.space())
                    .append(c_pret)
                    .group()
                    .parens()
            }
            LExpr::If(c, ift, iff) => {
                let c_pret = c.pretty(allocator);
                let ift_pret = ift.pretty(allocator);
                let iff_pret = iff.pretty(allocator);

                allocator
                    .text("if")
                    .append(allocator.space())
                    .append(c_pret)
                    .append(allocator.space())
                    .append(ift_pret)
                    .append(allocator.space())
                    .append(iff_pret)
                    .group()
                    .parens()
            }
            LExpr::Lifted(Ignore(l)) => allocator
                .text("lifted-lambda@")
                .append(allocator.as_string(l)),
            LExpr::CallOne(f, c) => {
                let f_pret = f.pretty(allocator);
                let c_pret = c.pretty(allocator);

                f_pret
                    .annotate(ColorSpec::new().set_fg(Some(Color::Blue)).clone())
                    .append(allocator.space())
                    .append(c_pret)
                    .group()
                    .parens()
            }
            LExpr::CallTwo(f, v, c) => {
                let f_pret = f.pretty(allocator);
                let v_pret = v.pretty(allocator);
                let c_pret = c.pretty(allocator);

                f_pret
                    .annotate(ColorSpec::new().set_fg(Some(Color::Blue)).clone())
                    .append(allocator.space())
                    .append(v_pret)
                    .append(allocator.space())
                    .append(c_pret)
                    .group()
                    .parens()
            }
        }
    }

    pub fn pretty_print(&self, out: impl WriteColor) -> Result<()> {
        let allocator = BoxAllocator;

        self.pretty(&allocator).1.render_colored(70, out)?;

        Ok(())
    }
}
