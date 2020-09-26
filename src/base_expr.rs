use moniker::Binder;
use moniker::{FreeVar, Ignore, Scope, Var};
use pretty::{BoxAllocator, DocAllocator, DocBuilder};
use std::collections::HashMap;
use termcolor::{Color, ColorSpec, WriteColor};

use std::{io::Result, rc::Rc};

use crate::expr::Expr;
use crate::literals::Literal;
use crate::utils::clone_rc;

// TODO: Type Families magick?

#[derive(Debug, Clone)]
pub enum BExpr {
    Var(String),
    Lit(Literal),
    BuiltinIdent(String),
    If(Rc<BExpr>, Rc<BExpr>, Rc<BExpr>),
    Set(String, Rc<BExpr>),
    Let(Vec<(String, BExpr)>, BExprBody),
    Lam(Vec<String>, BExprBody),
    App(Rc<BExpr>, Vec<BExpr>),
}

#[derive(Debug, Clone)]
pub enum BExprBodyExpr {
    Def(String, BExpr),
    Expr(BExpr),
}

#[derive(Debug, Clone)]
pub struct BExprBody(pub Vec<BExprBodyExpr>, pub Rc<BExpr>);

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
            BExpr::Let(bindings, body) => {
                let bindings_pret = allocator
                    .intersperse(
                        bindings.iter().map(|(n, e)| {
                            allocator
                                .text(n.to_owned())
                                .annotate(ColorSpec::new().set_fg(Some(Color::Magenta)).clone())
                                .append(allocator.line())
                                .append(e.pretty(allocator))
                                .group()
                                .parens()
                        }),
                        allocator.line(),
                    )
                    .parens();

                bindings_pret
                    .append(allocator.line())
                    .append(body.pretty(allocator))
                    .nest(1)
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

                allocator
                    .text("lambda")
                    .annotate(ColorSpec::new().set_fg(Some(Color::Magenta)).clone())
                    .append(allocator.space())
                    .append(pat_pret)
                    .append(allocator.line())
                    .append(body.pretty(allocator))
                    .nest(1)
                    .group()
                    .parens()
            }
            BExpr::If(c, ift, iff) => {
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

    pub fn rewrite<F: Fn(BExpr) -> BExpr>(self, f: &F) -> BExpr {
        let processed_children = match self {
            BExpr::Var(_) | BExpr::Lit(_) | BExpr::BuiltinIdent(_) => self,
            BExpr::Set(n, e) => BExpr::Set(n, Rc::new(clone_rc(e).rewrite(f))),
            BExpr::If(c, ift, iff) => BExpr::If(
                Rc::new(clone_rc(c).rewrite(f)),
                Rc::new(clone_rc(ift).rewrite(f)),
                Rc::new(clone_rc(iff).rewrite(f)),
            ),
            BExpr::Let(bindings, body) => {
                let new_bindings: Vec<_> = bindings
                    .into_iter()
                    .map(|(n, e)| (n, e.rewrite(f)))
                    .collect();

                BExpr::Let(new_bindings, body.rewrite(f))
            }
            BExpr::Lam(n, body) => BExpr::Lam(n, body.rewrite(f)),
            BExpr::App(r, es) => {
                let new_es = es.into_iter().map(|e| e.rewrite(f)).collect();

                BExpr::App(Rc::new(clone_rc(r).rewrite(f)), new_es)
            }
        };

        f(processed_children)
    }

    pub fn remove_let(self) -> BExpr {
        fn t(e: BExpr) -> BExpr {
            match e {
                BExpr::Let(b, e) => {
                    let (names, params) = b.into_iter().unzip();

                    let lam = Rc::new(BExpr::Lam(names, e));
                    BExpr::App(lam, params)
                }
                _ => e,
            }
        }

        self.rewrite(&t)
    }

    pub fn lift_defines(self) -> BExpr {
        fn t(e: BExpr) -> BExpr {
            match e {
                BExpr::Let(e, body) => BExpr::Let(e, body.pull_defines()),
                BExpr::Lam(e, body) => BExpr::Lam(e, body.pull_defines()),
                _ => e,
            }
        }

        self.rewrite(&t)
    }

    pub fn into_expr(self) -> Expr {
        let env = HashMap::new();
        self.lift_defines().remove_let().into_expr_inner(&env)
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
                let body = body.as_expressions();

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
            BExpr::If(c, ift, iff) => {
                let c = clone_rc(c).into_expr_inner(env);
                let ift = clone_rc(ift).into_expr_inner(env);
                let iff = clone_rc(iff).into_expr_inner(env);

                Expr::If(Rc::new(c), Rc::new(ift), Rc::new(iff))
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
            BExpr::Let(_, _) => unreachable!(),
        }
    }
}

impl BExprBodyExpr {
    pub fn pretty<'a, D>(&self, allocator: &'a D) -> DocBuilder<'a, D, ColorSpec>
    where
        D: DocAllocator<'a, ColorSpec>,
        D::Doc: Clone,
    {
        match self {
            BExprBodyExpr::Def(n, e) => {
                let e_pret = e.pretty(allocator);

                allocator
                    .text("define")
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
            BExprBodyExpr::Expr(e) => e.pretty(allocator),
        }
    }

    pub fn rewrite<F: Fn(BExpr) -> BExpr>(self, f: &F) -> BExprBodyExpr {
        match self {
            BExprBodyExpr::Def(n, e) => BExprBodyExpr::Def(n, e.rewrite(f)),
            BExprBodyExpr::Expr(e) => BExprBodyExpr::Expr(e.rewrite(f)),
        }
    }
}

impl BExprBody {
    pub fn pretty<'a, D>(&self, allocator: &'a D) -> DocBuilder<'a, D, ColorSpec>
    where
        D: DocAllocator<'a, ColorSpec>,
        D::Doc: Clone,
    {
        allocator.intersperse(
            self.0
                .iter()
                .map(|e| e.pretty(allocator))
                .chain(std::iter::once(self.1.pretty(allocator))),
            allocator.hardline(),
        )
    }

    pub fn rewrite<F: Fn(BExpr) -> BExpr>(self, f: &F) -> BExprBody {
        BExprBody(
            self.0.into_iter().map(|e| e.rewrite(f)).collect(),
            Rc::new(clone_rc(self.1).rewrite(f)),
        )
    }

    pub fn pull_defines(self) -> BExprBody {
        let mut defines = Vec::new();

        let body = self
            .0
            .into_iter()
            .map(|e| match e {
                BExprBodyExpr::Def(n, e) => {
                    defines.push(n.clone());
                    BExprBodyExpr::Expr(BExpr::Set(n, Rc::new(e)))
                }
                BExprBodyExpr::Expr(_) => e,
            })
            .collect();

        let let_bindings = defines
            .into_iter()
            .map(|n| (n, BExpr::Lit(Literal::Void)))
            .collect();
        let let_ = BExpr::Let(let_bindings, BExprBody(body, self.1));

        BExprBody(Vec::new(), Rc::new(let_))
    }

    pub fn as_expressions(self) -> Vec<BExpr> {
        self.0
            .into_iter()
            .map(|e| match e {
                BExprBodyExpr::Def(_, _) => panic!("should be removed"),
                BExprBodyExpr::Expr(e) => e,
            })
            .chain(std::iter::once(clone_rc(self.1)))
            .collect()
    }
}
