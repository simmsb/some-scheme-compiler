use moniker::Binder;
use moniker::{FreeVar, Ignore, Scope, Var};
use pretty::{BoxAllocator, DocAllocator, DocBuilder};
use std::collections::HashMap;
use termcolor::{Color, ColorSpec, WriteColor};

use std::{io::Result, rc::Rc};

use crate::expr::Expr;
use crate::literals::Literal;
use crate::utils::clone_rc;

#[derive(Debug, Clone)]
pub enum BExpr {
    Var(String),
    Lit(Literal),
    BuiltinIdent(String),
    Set(String, Rc<BExpr>),
    Lam(Vec<String>, Vec<BExpr>),
    App(Rc<BExpr>, Vec<BExpr>),
}

impl BExpr {
    pub fn pretty<'a, D>(&self, allocator: &'a D) -> DocBuilder<'a, D, ColorSpec>
    where
        D: DocAllocator<'a, ColorSpec>,
        D::Doc: Clone,
    {
        match self {
            BExpr::Var(s) => allocator.as_string(s),
            BExpr::Lit(l) => l.pretty(allocator),
            BExpr::BuiltinIdent(s) => allocator.as_string(s),
            BExpr::Set(n, e) => {
                let e_pret = e.pretty(allocator);

                allocator
                    .text("set!")
                    .annotate(ColorSpec::new().set_fg(Some(Color::Magenta)).clone())
                    .append(allocator.space())
                    .append(
                        allocator
                            .text(n.to_owned())
                            .annotate(ColorSpec::new().set_fg(Some(Color::Green)).clone()),
                    )
                    .append(allocator.space())
                    .append(e_pret)
                    .group()
                    .parens()
            }
            BExpr::Lam(pat, body) => {
                let pat_pret = allocator
                    .intersperse(
                        pat.iter().map(|p| {
                            allocator
                                .text(p.to_owned())
                                .annotate(ColorSpec::new().set_fg(Some(Color::Green)).clone())
                        }),
                        allocator.line(),
                    )
                    .parens();

                let body_pret = allocator.intersperse(
                    body.iter().map(|e| e.pretty(allocator)),
                    allocator.hardline(),
                );

                allocator
                    .text("lambda")
                    .annotate(ColorSpec::new().set_fg(Some(Color::Magenta)).clone())
                    .append(allocator.space())
                    .append(pat_pret)
                    .append(allocator.line())
                    .append(body_pret)
                    .nest(1)
                    .group()
                    .parens()
            }
            BExpr::App(f, params) => {
                let f_pret = f.pretty(allocator);
                let v_pret = allocator
                    .intersperse(params.iter().map(|v| v.pretty(allocator)), allocator.line());

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

    pub fn into_expr(self) -> Expr {
        let env = HashMap::new();
        self.into_expr_inner(&env)
    }

    fn into_expr_inner(self, env: &HashMap<String, FreeVar<String>>) -> Expr {
        match self {
            BExpr::Var(n) => Expr::Var(Var::Free(
                env.get(&n)
                    .unwrap_or_else(|| panic!("unbound arg: {}", n))
                    .clone(),
            )),
            BExpr::Lit(l) => Expr::Lit(Ignore(l)),
            BExpr::BuiltinIdent(l) => Expr::BuiltinIdent(Ignore(l)),
            BExpr::Set(n, e) => Expr::Set(
                Var::Free(
                    env.get(&n)
                        .unwrap_or_else(|| panic!("unbound arg: {}", n))
                        .clone(),
                ),
                Rc::new(clone_rc(e).into_expr_inner(env)),
            ),
            BExpr::Lam(params, body) => {
                let mut env = env.clone();
                env.extend(params.iter().map(|n| (n.clone(), FreeVar::fresh_named(n))));

                let body = match body.as_slice() {
                    [] => Expr::Lit(Ignore(Literal::Void)),
                    [first, rest @ ..] => {
                        rest.iter()
                            .fold(first.clone().into_expr_inner(&env), |acc, e| {
                                Expr::App(
                                    Rc::new(Expr::Lam(Scope::new(
                                        Binder(FreeVar::fresh_named("_unused")),
                                        Rc::new(e.clone().into_expr_inner(&env)),
                                    ))),
                                    Rc::new(acc),
                                )
                            })
                    }
                };

                match params.as_slice() {
                    [] => {
                        // for zero param functions, we turn it into a 1-param function
                        // and then on calls with zero parameters, we add in a parameter of null
                        Expr::Lam(Scope::new(
                            Binder(FreeVar::fresh_named("_unused")),
                            Rc::new(body),
                        ))
                    }
                    [rest @ .., last] => {
                        let last = Expr::Lam(Scope::new(
                            Binder(env.get(last).unwrap().clone()),
                            Rc::new(body),
                        ));
                        rest.iter().rev().fold(last, |acc, p| {
                            Expr::Lam(Scope::new(
                                Binder(env.get(p).unwrap().clone()),
                                Rc::new(acc),
                            ))
                        })
                    }
                }
            }
            BExpr::App(expr, params) => {
                let expr = clone_rc(expr).into_expr_inner(env);

                match params.as_slice() {
                    [] => Expr::App(Rc::new(expr), Rc::new(Expr::Lit(Ignore(Literal::Void)))),
                    args => args.iter().fold(expr, |acc, p| {
                        Expr::App(Rc::new(acc), Rc::new(p.clone().into_expr_inner(env)))
                    }),
                }
            }
        }
    }
}
