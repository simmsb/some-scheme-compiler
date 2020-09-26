use moniker::BoundTerm;
use moniker::{Binder, Ignore, Scope, Var};

use pretty::{BoxAllocator, DocAllocator, DocBuilder};
use termcolor::{Color, ColorSpec, WriteColor};

use std::{io::Result, rc::Rc};

use crate::cont_expr;
use crate::flat_expr;
use crate::literals::Literal;

#[derive(Debug, Clone, BoundTerm)]
pub enum Expr {
    Var(Var<String>),
    Lit(Ignore<Literal>),
    BuiltinIdent(Ignore<String>),
    If(Rc<Expr>, Rc<Expr>, Rc<Expr>),
    Set(Var<String>, Rc<Expr>),
    Lam(Scope<Binder<String>, Rc<Expr>>),
    App(Rc<Expr>, Rc<Expr>),
}

impl Expr {
    pub fn pretty<'a, D>(&self, allocator: &'a D) -> DocBuilder<'a, D, ColorSpec>
    where
        D: DocAllocator<'a, ColorSpec>,
        D::Doc: Clone,
    {
        match self {
            Expr::Var(s) => allocator.as_string(s),
            Expr::Lit(Ignore(l)) => l.pretty(allocator),
            Expr::BuiltinIdent(Ignore(s)) => allocator.as_string(s),
            Expr::Set(s, e) => {
                let e_pret = e.pretty(allocator);

                allocator
                    .text("set!")
                    .annotate(ColorSpec::new().set_fg(Some(Color::Magenta)).clone())
                    .append(allocator.space())
                    .append(
                        allocator
                            .as_string(s)
                            .annotate(ColorSpec::new().set_fg(Some(Color::Green)).clone()),
                    )
                    .append(allocator.space())
                    .append(e_pret)
                    .group()
                    .parens()
            }
            Expr::Lam(s) => {
                let Scope {
                    unsafe_pattern: pat,
                    unsafe_body: body,
                } = &s;

                let pat_pret = allocator
                    .as_string(pat)
                    .annotate(ColorSpec::new().set_fg(Some(Color::Green)).clone())
                    .parens();
                let body_pret = allocator.line_().append(body.pretty(allocator));

                allocator
                    .text("lambda")
                    .annotate(ColorSpec::new().set_fg(Some(Color::Magenta)).clone())
                    .append(allocator.space())
                    .append(pat_pret)
                    .append(allocator.space())
                    .append(body_pret)
                    .nest(1)
                    .group()
                    .parens()
            }
            Expr::If(c, ift, iff) => {
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
            Expr::App(f, v) => {
                let f_pret = f.pretty(allocator);
                let v_pret = v.pretty(allocator);

                f_pret
                    .annotate(ColorSpec::new().set_fg(Some(Color::Blue)).clone())
                    .append(allocator.space())
                    .append(v_pret)
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

    pub fn into_fexpr(self, k: Rc<cont_expr::AExp>) -> flat_expr::FExpr {
        cont_expr::t_c(self, k).into_fexpr()
    }
}
