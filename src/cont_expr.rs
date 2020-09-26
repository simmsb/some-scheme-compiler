use moniker::BoundTerm;
use moniker::{Binder, FreeVar, Ignore, Scope, Var};

use pretty::{BoxAllocator, DocAllocator, DocBuilder};
use termcolor::{Color, ColorSpec, WriteColor};

use std::{io::Result, rc::Rc};

use crate::{expr::Expr, flat_expr::FExpr, literals::Literal, utils::clone_rc};

#[derive(Debug, Clone, BoundTerm)]
pub enum AExp {
    Lam2(Scope<Binder<String>, Scope<Binder<String>, Rc<CExp>>>),
    Lam1(Scope<Binder<String>, Rc<CExp>>),
    Var(Var<String>),
    BuiltinIdent(Ignore<String>),
    Lit(Ignore<Literal>),
}

impl AExp {
    pub fn pretty<'a, D>(&self, allocator: &'a D) -> DocBuilder<'a, D, ColorSpec>
    where
        D: DocAllocator<'a, ColorSpec>,
        D::Doc: Clone,
    {
        match self {
            AExp::Lam2(s) => {
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
                let body_pret = allocator.line_().append(body.pretty(allocator));

                allocator
                    .text("lambda")
                    .annotate(ColorSpec::new().set_fg(Some(Color::Magenta)).clone())
                    .append(allocator.space())
                    .append(args_pret)
                    .append(allocator.space())
                    .append(body_pret)
                    .nest(1)
                    .group()
                    .parens()
            }
            AExp::Lam1(s) => {
                let Scope {
                    unsafe_pattern: pat,
                    unsafe_body: body,
                } = &s;

                let pat_pret = allocator
                    .as_string(pat)
                    .annotate(ColorSpec::new().set_fg(Some(Color::Green)).clone());
                let args_pret = pat_pret.append(allocator.space()).parens();
                let body_pret = allocator.line_().append(body.pretty(allocator));

                allocator
                    .text("lambda")
                    .annotate(ColorSpec::new().set_fg(Some(Color::Magenta)).clone())
                    .append(allocator.space())
                    .append(args_pret)
                    .append(allocator.space())
                    .append(body_pret)
                    .nest(1)
                    .group()
                    .parens()
            }
            AExp::Var(s) => allocator.as_string(s),
            AExp::BuiltinIdent(Ignore(i)) => allocator.as_string(i),
            AExp::Lit(Ignore(l)) => l.pretty(allocator),
        }
    }

    pub fn into_fexpr(self) -> FExpr {
        match self {
            AExp::Lam2(s) => {
                let (pat, body) = s.unbind();
                let (cont, body) = body.unbind();

                FExpr::LamTwo(Scope::new(
                    pat,
                    Scope::new(cont, Rc::new(clone_rc(body).into_fexpr())),
                ))
            }
            AExp::Lam1(s) => {
                let (pat, body) = s.unbind();

                FExpr::LamOne(Scope::new(pat, Rc::new(clone_rc(body).into_fexpr())))
            }
            AExp::BuiltinIdent(s) => FExpr::BuiltinIdent(s),
            AExp::Var(s) => FExpr::Var(s),
            AExp::Lit(l) => FExpr::Lit(l),
        }
    }
}

#[derive(Debug, Clone, BoundTerm)]
pub enum CExp {
    If(Rc<AExp>, Rc<CExp>, Rc<CExp>),
    SetThen(Var<String>, Rc<AExp>, Rc<CExp>),
    Call1(Rc<AExp>, Rc<AExp>),
    Call2(Rc<AExp>, Rc<AExp>, Rc<AExp>),
}

impl CExp {
    pub fn pretty<'a, D>(&self, allocator: &'a D) -> DocBuilder<'a, D, ColorSpec>
    where
        D: DocAllocator<'a, ColorSpec>,
        D::Doc: Clone,
    {
        match self {
            CExp::If(c, ift, iff) => {
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

            CExp::SetThen(n, v, c) => {
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

            CExp::Call2(f, v, c) => {
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

            CExp::Call1(f, c) => {
                let f_pret = f.pretty(allocator);
                let c_pret = c.pretty(allocator);

                f_pret
                    .annotate(ColorSpec::new().set_fg(Some(Color::Blue)).clone())
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

    pub fn into_fexpr(self) -> FExpr {
        match self {
            CExp::SetThen(n, v, c) => FExpr::SetThen(
                n,
                Rc::new(clone_rc(v).into_fexpr()),
                Rc::new(clone_rc(c).into_fexpr()),
            ),
            CExp::Call2(f, v, c) => FExpr::CallTwo(
                Rc::new(clone_rc(f).into_fexpr()),
                Rc::new(clone_rc(v).into_fexpr()),
                Rc::new(clone_rc(c).into_fexpr()),
            ),
            CExp::Call1(f, v) => FExpr::CallOne(
                Rc::new(clone_rc(f).into_fexpr()),
                Rc::new(clone_rc(v).into_fexpr()),
            ),
            CExp::If(c, ift, iff) => FExpr::If(
                Rc::new(clone_rc(c).into_fexpr()),
                Rc::new(clone_rc(ift).into_fexpr()),
                Rc::new(clone_rc(iff).into_fexpr()),
            ),
        }
    }
}

fn t_k(expr: Expr, fk: &dyn Fn(Rc<AExp>) -> CExp) -> CExp {
    match expr {
        Expr::Lam(_) | Expr::Var(_) | Expr::Lit(_) | Expr::BuiltinIdent(_) => fk(Rc::new(m(expr))),
        Expr::Set(n, e) => t_k(clone_rc(e), &|e| {
            CExp::SetThen(
                n.clone(),
                e,
                Rc::new(fk(Rc::new(AExp::Lit(Ignore(Literal::Void))))),
            )
        }),
        Expr::If(c, ift, iff) => {
            let rv_v = FreeVar::fresh_named("rv");
            let cont = Rc::new(AExp::Lam1(Scope::new(
                Binder(rv_v.clone()),
                Rc::new(fk(Rc::new(AExp::Var(Var::Free(rv_v))))),
            )));
            t_k(clone_rc(c), &|c| {
                CExp::If(
                    c,
                    Rc::new(t_c(clone_rc(ift.clone()), cont.clone())),
                    Rc::new(t_c(clone_rc(iff.clone()), cont.clone())),
                )
            })
        }
        Expr::App(f, e) => {
            let rv_v = FreeVar::fresh_named("rv");
            let cont = Rc::new(AExp::Lam1(Scope::new(
                Binder(rv_v.clone()),
                Rc::new(fk(Rc::new(AExp::Var(Var::Free(rv_v))))),
            )));

            t_k(clone_rc(f), &|f| {
                t_k(clone_rc(e.clone()), &|e| {
                    CExp::Call2(f.clone(), e, cont.clone())
                })
            })
        }
    }
}

pub fn t_c(expr: Expr, c: Rc<AExp>) -> CExp {
    match expr {
        e @ (Expr::Lam(_) | Expr::Var(_) | Expr::Lit(_) | Expr::BuiltinIdent(_)) => {
            CExp::Call1(c, Rc::new(m(e)))
        }
        Expr::Set(n, e) => t_k(clone_rc(e), &|e| {
            CExp::SetThen(
                n.clone(),
                e,
                Rc::new(CExp::Call1(
                    c.clone(),
                    Rc::new(AExp::Lit(Ignore(Literal::Void))),
                )),
            )
        }),
        Expr::If(cond, ift, iff) => {
            let k = FreeVar::fresh_named("k");
            CExp::Call1(
                Rc::new(AExp::Lam1(Scope::new(
                    Binder(k.clone()),
                    Rc::new(t_k(clone_rc(cond.clone()), &|cond| {
                        CExp::If(
                            cond,
                            Rc::new(t_c(
                                clone_rc(ift.clone()),
                                Rc::new(AExp::Var(Var::Free(k.clone()))),
                            )),
                            Rc::new(t_c(
                                clone_rc(iff.clone()),
                                Rc::new(AExp::Var(Var::Free(k.clone()))),
                            )),
                        )
                    })),
                ))),
                c,
            )
        }
        Expr::App(f, e) => t_k(clone_rc(f), &|f| {
            t_k(e.as_ref().clone(), &|e| {
                CExp::Call2(f.clone(), e, c.clone())
            })
        }),
    }
}

pub fn m(expr: Expr) -> AExp {
    match expr {
        Expr::Lam(s) => {
            let (p, t) = s.unbind();
            let k = FreeVar::fresh_named("k");
            let body = t_c(clone_rc(t), Rc::new(AExp::Var(Var::Free(k.clone()))));
            AExp::Lam2(Scope::new(p, Scope::new(Binder(k), Rc::new(body))))
        }
        Expr::Var(v) => AExp::Var(v),
        Expr::BuiltinIdent(v) => AExp::BuiltinIdent(v),
        Expr::Lit(v) => AExp::Lit(v),
        _ => unreachable!(),
    }
}
