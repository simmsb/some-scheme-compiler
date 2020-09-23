use moniker::BoundTerm;
use moniker::{Binder, FreeVar, Ignore, Scope, Var};

use pretty::{BoxAllocator, DocAllocator, DocBuilder};
use termcolor::{Color, ColorSpec, WriteColor};

use std::{io::Result, rc::Rc};

use crate::{expr::Expr, flat_expr::FExpr, literals::Literal, utils::clone_rc};

#[derive(Debug, Clone, BoundTerm)]
pub enum UExpr {
    Lam(Scope<Binder<String>, Scope<Binder<String>, Rc<CCall>>>),
    Var(Var<String>),
    BuiltinIdent(Ignore<String>),
    Lit(Ignore<Literal>),
}

impl UExpr {
    pub fn pretty<'a, D>(&self, allocator: &'a D) -> DocBuilder<'a, D, ColorSpec>
    where
        D: DocAllocator<'a, ColorSpec>,
        D::Doc: Clone,
    {
        match self {
            UExpr::Lam(s) => {
                let Scope {
                    unsafe_pattern: pat,
                    unsafe_body:
                        Scope {
                            unsafe_pattern: cont,
                            unsafe_body: body,
                        },
                } = &s;

                let pat_pret = allocator
                    .as_string(pat)
                    .annotate(ColorSpec::new().set_fg(Some(Color::Green)).clone());
                let cont_pret = allocator
                    .as_string(cont)
                    .annotate(ColorSpec::new().set_fg(Some(Color::Red)).clone());
                let args_pret = pat_pret
                    .append(allocator.space())
                    .append(cont_pret)
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
                    .append(args_pret)
                    .append(allocator.space())
                    .append(body_pret)
                    .parens()
            }
            UExpr::Var(s) => allocator.as_string(s),
            UExpr::BuiltinIdent(Ignore(i)) => allocator.as_string(i),
            UExpr::Lit(Ignore(l)) => l.pretty(allocator),
        }
    }

    pub fn into_fexpr(self) -> FExpr {
        match self {
            UExpr::Lam(s) => {
                let (pat, body) = s.unbind();
                let (cont, body) = body.unbind();

                FExpr::LamTwo(Scope::new(
                    pat,
                    Scope::new(cont, Rc::new(clone_rc(body).into_fexpr())),
                ))
            }
            UExpr::BuiltinIdent(s) => FExpr::BuiltinIdent(s),
            UExpr::Var(s) => FExpr::Var(s),
            UExpr::Lit(l) => FExpr::Lit(l),
        }
    }
}

#[derive(Debug, Clone, BoundTerm)]
pub enum KExpr {
    Lam(Scope<Binder<String>, Rc<CCall>>),
    Var(Var<String>),
    BuiltinIdent(Ignore<String>),
    Lit(Ignore<Literal>),
}

impl KExpr {
    pub fn pretty<'a, D>(&self, allocator: &'a D) -> DocBuilder<'a, D, ColorSpec>
    where
        D: DocAllocator<'a, ColorSpec>,
        D::Doc: Clone,
    {
        match self {
            KExpr::Lam(s) => {
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
            KExpr::Var(s) => allocator.as_string(s),
            KExpr::BuiltinIdent(Ignore(i)) => allocator.as_string(i),
            KExpr::Lit(Ignore(l)) => l.pretty(allocator),
        }
    }

    pub fn into_fexpr(self) -> FExpr {
        match self {
            KExpr::Lam(s) => {
                let (pat, body) = s.unbind();
                FExpr::LamOne(Scope::new(pat, Rc::new(clone_rc(body).into_fexpr())))
            }
            KExpr::Var(s) => FExpr::Var(s),
            KExpr::BuiltinIdent(s) => FExpr::BuiltinIdent(s),
            KExpr::Lit(l) => FExpr::Lit(l),
        }
    }
}

#[derive(Debug, Clone, BoundTerm)]
pub enum CCall {
    UCall(Rc<UExpr>, Rc<UExpr>, Rc<KExpr>),
    KCall(Rc<KExpr>, Rc<UExpr>),
}

impl CCall {
    pub fn pretty<'a, D>(&self, allocator: &'a D) -> DocBuilder<'a, D, ColorSpec>
    where
        D: DocAllocator<'a, ColorSpec>,
        D::Doc: Clone,
    {
        match self {
            CCall::UCall(f, v, c) => {
                let f_pret = f.pretty(allocator);
                let v_pret = v.pretty(allocator);
                let c_pret = c.pretty(allocator);

                f_pret
                    .annotate(ColorSpec::new().set_fg(Some(Color::Blue)).clone())
                    .append(allocator.space())
                    .append(v_pret)
                    .append(allocator.space())
                    .append(c_pret)
                    .parens()
            }

            CCall::KCall(f, c) => {
                let f_pret = f.pretty(allocator);
                let c_pret = c.pretty(allocator);

                f_pret
                    .annotate(ColorSpec::new().set_fg(Some(Color::Blue)).clone())
                    .append(allocator.space())
                    .append(c_pret)
                    .parens()
            }
        }
    }

    pub fn pretty_print(&self, out: impl WriteColor) -> Result<()> {
        let allocator = BoxAllocator;

        self.pretty(&allocator).1.render_colored(70, out)?;

        Ok(())
    }

    pub fn into_fexpr(self) -> FExpr {
        match self {
            CCall::UCall(f, v, c) => FExpr::CallTwo(
                Rc::new(clone_rc(f).into_fexpr()),
                Rc::new(clone_rc(v).into_fexpr()),
                Rc::new(clone_rc(c).into_fexpr()),
            ),
            CCall::KCall(f, v) => FExpr::CallOne(
                Rc::new(clone_rc(f).into_fexpr()),
                Rc::new(clone_rc(v).into_fexpr()),
            ),
        }
    }
}

fn t_k(expr: Expr, fk: &dyn Fn(Rc<UExpr>) -> CCall) -> CCall {
    match expr {
        Expr::Lam(_) | Expr::Var(_) | Expr::Lit(_) | Expr::BuiltinIdent(_) => fk(Rc::new(m(expr))),
        Expr::App(f, e) => {
            let rv_v = FreeVar::fresh_named("rv");
            let cont = Rc::new(KExpr::Lam(Scope::new(
                Binder(rv_v.clone()),
                Rc::new(fk(Rc::new(UExpr::Var(Var::Free(rv_v))))),
            )));

            t_k(clone_rc(f), &|f| {
                t_k(clone_rc(e.clone()), &|e| CCall::UCall(f.clone(), e.clone(), cont.clone()))
            })
        }
    }
}

pub fn t_c(expr: Expr, c: Rc<KExpr>) -> CCall {
    match expr {
        e @ (Expr::Lam(_) | Expr::Var(_) | Expr::Lit(_) | Expr::BuiltinIdent(_)) => {
            CCall::KCall(c, Rc::new(m(e)))
        }
        Expr::App(f, e) => t_k(clone_rc(f), &|f| {
            t_k(clone_rc(e.clone()), &|e| CCall::UCall(f.clone(), e, c.clone()))
        }),
    }
}

pub fn m(expr: Expr) -> UExpr {
    match expr {
        Expr::Lam(s) => {
            let (p, t) = s.unbind();
            let k = FreeVar::fresh_named("k");
            let body = t_c(clone_rc(t), Rc::new(KExpr::Var(Var::Free(k.clone()))));
            UExpr::Lam(Scope::new(p, Scope::new(Binder(k), Rc::new(body))))
        }
        Expr::Var(v) => UExpr::Var(v),
        Expr::BuiltinIdent(v) => UExpr::BuiltinIdent(v),
        Expr::Lit(v) => UExpr::Lit(v),
        _ => unreachable!(),
    }
}
