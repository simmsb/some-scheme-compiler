use moniker::BoundTerm;
use moniker::{Binder, Scope, Var, Ignore};

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
            Expr::Lam(s) => {
                let Scope {
                    unsafe_pattern: pat,
                    unsafe_body: body,
                } = &s;
               
                let pat_pret = allocator
                    .as_string(pat)
                    .annotate(ColorSpec::new().set_fg(Some(Color::Green)).clone())
                    .parens();
                let body_pret = allocator
                    .line_()
                    .append(body.pretty(allocator))
                    .nest(1)
                    .group();

                allocator
                    .text("lambda")
                    .annotate(ColorSpec::new().set_fg(Some(Color::Magenta)).clone())
                    .append(allocator.space())
                    .append(pat_pret)
                    .append(allocator.space())
                    .append(body_pret)
                    .parens()
            }
            Expr::App(f, v) => {
                let f_pret = f.pretty(allocator);
                let v_pret = v.pretty(allocator);

                f_pret
                    .annotate(ColorSpec::new().set_fg(Some(Color::Blue)).clone())
                    .append(allocator.space())
                    .append(v_pret)
                    .parens()
            }
        }
    }

    pub fn pretty_print(&self, out: impl WriteColor) -> Result<()> {
        let allocator = BoxAllocator;

        self.pretty(&allocator).1.render_colored(70, out)?;

        Ok(())
    }

    pub fn into_fexpr(self, k: Rc<cont_expr::KExpr>) -> flat_expr::FExpr {
        cont_expr::t_k(self, k).into_fexpr()
    }
}
