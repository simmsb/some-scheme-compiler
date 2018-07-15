use std::borrow::Cow;

use nodes::{LExpr, ExprLit};

// compiler transformation stage

pub struct TransformContext {
    genvar_count: u64,
}

impl TransformContext {
    pub fn new() -> Self {
        TransformContext { genvar_count: 0 }
    }

    pub fn gen_ident<'a>(&mut self, name: &str) -> Cow<'a, str> {
        let var = format!("$anon_var_{}_{}", name, self.genvar_count);
        self.genvar_count += 1;
        Cow::from(var)
    }

    pub fn gen_var<'a>(&mut self, name: &str) -> LExpr<'a> {
        LExpr::Var(self.gen_ident(name))
    }

    pub fn gen_cont<'a>(&mut self) -> LExpr<'a> {
        let var = format!("$cont_var_{}", self.genvar_count);
        self.genvar_count += 1;
        LExpr::Var(Cow::from(var))
    }

    pub fn gen_throwaway<'a>(&mut self) -> Cow<'a, str> {
        let var = format!("$throwaway_var_{}", self.genvar_count);
        self.genvar_count += 1;
        Cow::from(var)
    }

    pub fn gen_throwaway_var<'a>(&mut self) -> LExpr<'a> {
        LExpr::Var(self.gen_throwaway())
    }
}

fn void_obj() -> LExpr<'static> {
    LExpr::Lit(ExprLit::Void)
}


/// Renames some functions to their builtin equivalent
///
/// For example:
/// ```scheme
/// (+ 1 2)
/// ```
///
/// becomes
///
/// ```scheme
/// (builtin_int_obj_add_func_1 1 2)
/// ```
pub fn rename_builtins<'a>(expr: LExpr<'a>, ctx: &mut TransformContext) -> LExpr<'a> {
    use nodes::LExpr::*;

    match expr {
        Lam(args, body) => {
            let body: Vec<_> = body
                .into_iter()
                .map(|e| rename_builtins(e, ctx))
                .collect();
            Lam(args, body)
        },
        App(box operator, operands) => {
            let operator = rename_builtins(operator, ctx);
            let operands: Vec<_> = operands
                .into_iter()
                .map(|e| rename_builtins(e, ctx))
                .collect();
            App(box operator, operands)
        }
        Var(var) => {
            let builtin_name = match var.as_ref() {
                "+" => "object_int_obj_add",
                "-" => "object_int_obj_sub",
                "*" => "object_int_obj_mul",
                "/" => "object_int_obj_div",
                _   => return Var(var),
            };
            BuiltinIdent(Cow::from(builtin_name))
        },
        Lit(..) | BuiltinIdent(..) | BuiltinApp(..) => expr,
        _ => unreachable!("Shouldn't be touching this yet."),
    }
}


/// Transforms literals into their correct literal constructor form
///
/// For example:
/// ```scheme
/// 12
/// ```
///
/// becomes
///
/// ```scheme
/// (object_int_obj_new 12)
/// ```
pub fn transform_lits<'a>(expr: LExpr<'a>, ctx: &mut TransformContext) -> LExpr<'a> {
    use nodes::LExpr::*;

    match expr {
        Lam(args, body) => {
            let body: Vec<_> = body
                .into_iter()
                .map(|e| transform_lits(e, ctx))
                .collect();
            Lam(args, body)
        },
        App(box operator, operands) => {
            let operator = transform_lits(operator, ctx);
            let operands: Vec<_> = operands
                .into_iter()
                .map(|e| transform_lits(e, ctx))
                .collect();
            App(box operator, operands)
        }
        Var(..) | BuiltinIdent(..) => expr,
        Lit(lit) => {
            // special case for void type which has no param
            if let ExprLit::Void = lit {
                let fn_name = BuiltinIdent(Cow::from("object_void_obj_new"));
                return App(box fn_name, vec![]);
            };

            let ctor_fn = match lit {
                ExprLit::StringLit(..) => "object_str_obj_new",
                ExprLit::NumLit(..)    => "object_int_obj_new",
                ExprLit::Void          => unreachable!(),
            };

            let ctor_fn = Cow::from(ctor_fn);

            BuiltinApp(ctor_fn, box Lit(lit))
        },
        _ => unreachable!("Shouldn't be touching this yet."),
    }
}


/// Transform multiple parameter lambdas into nested single parmeters.
///
/// ```scheme
/// (lambda (a b c) ...)
/// becomes
/// (lambda (a)
///   (lambda (b)
///     (lambda (c)
///       ...)))
/// ```
///
/// Transform calls with multiple parameters into nested calls each applying single parameters
///
/// ```scheme
/// (f a b c)
/// ```
///
/// becomes
///
/// ```scheme
/// ((((f) a) b) c)
/// ```
pub fn expand_lam_app<'a>(expr: LExpr<'a>, ctx: &mut TransformContext) -> LExpr<'a> {
    use nodes::LExpr::*;

    match expr {
        Lam(args, body) => {
            let body: Vec<_> = body
                .into_iter()
                .map(|x| expand_lam_app(x, ctx))
                .collect();
            match args.len() {
                0 => LamOne(ctx.gen_throwaway(), body),
                _ => {
                    let mut iter = args.into_iter().rev();

                    let first = LamOne(iter.next().unwrap(), body);

                    iter.fold(first, |acc, arg| LamOne(arg, vec![acc]))
                }
            }
        }
        App(box operator, operands) => {
            let operator = expand_lam_app(operator, ctx);
            let operands: Vec<_> = operands
                .into_iter()
                .map(|o| expand_lam_app(o, ctx))
                .collect();
            let num_operands = operands.len();
            match num_operands {
                0 => AppOne(box operator, box void_obj()),
                _ => {
                    let mut operands = operands.into_iter();

                    let first = AppOne(box operator, box operands.next().unwrap());

                    operands.fold(first, |acc, arg| AppOne(box acc, box arg))
                }
            }
        }
        Var(..) | Lit(..) | BuiltinIdent(..) | BuiltinApp(..) => expr,
        _ => unreachable!("Shouldn't be touching this yet"),
    }
}

/// Transform body of lambda from multiple expressions into a single expression
///
/// (lambda () a b c)
///
/// becomes:
///
/// (lambda ()
///  ((lambda ($unique) c)
///   ((lambda ($unique) b)
///    a)))
///
/// where $unique is a unique variable name
pub fn expand_lam_body<'a>(expr: LExpr<'a>, ctx: &mut TransformContext) -> LExpr<'a> {
    use nodes::LExpr::*;

    match expr {
        LamOne(arg, body) => {
            let num_body = body.len();
            let body: Vec<_> = body
                .into_iter()
                .rev()
                .map(|b| expand_lam_body(b, ctx))
                .collect();
            let inner = match num_body {
                0 => LamOneOne(arg.clone(), box void_obj()),
                _ => {
                    // get the last expression, as this won't be placed in a (... x) wrapper
                    let mut body = body.into_iter();
                    let first = body.next().unwrap();

                    body.fold(first, |acc, arg| {
                        AppOne(box LamOneOne(ctx.gen_ident("lam_expand"), box acc), box arg)
                    })
                }
            };
            LamOneOne(arg.clone(), box inner)
        }
        AppOne(box operator, box operand) => AppOne(
            box expand_lam_body(operator, ctx),
            box expand_lam_body(operand, ctx),
        ),
        x => x,
    }
}

/// Apply the cps transformation with a continuation expression
pub fn cps_transform_cont<'a>(
    expr: LExpr<'a>,
    cont: LExpr<'a>,
    ctx: &mut TransformContext,
) -> LExpr<'a> {
    match expr {
        LExpr::Var(..) | LExpr::LamOneOne(..) | LExpr::LamOneOneCont(..) | LExpr::BuiltinIdent(..) | LExpr::BuiltinApp(..) => {
            LExpr::AppOne(box cont, box cps_transform(expr, ctx))
        }
        LExpr::AppOne(box operator, box operand) => {
            let operator_var: Cow<'a, str> = ctx.gen_ident("operator_var");
            let operand_var: Cow<'a, str> = ctx.gen_ident("operand_var");
            let rv_var: Cow<'a, str> = ctx.gen_ident("rv");

            let new_cont = LExpr::LamOneOne(
                rv_var.clone(),
                box LExpr::AppOne(box cont.clone(), box LExpr::Var(rv_var.clone())),
            );

            // The expression:
            // (rator rand)
            // Is transformed into:
            // (T rator '(lambda (rator_var)
            //             (T rand '(lambda (rand_var)
            //                        (rator_var rand_var cont)))))
            cps_transform_cont(
                operator,
                LExpr::LamOneOne(
                    operator_var.clone(),
                    box cps_transform_cont(
                        operand,
                        LExpr::LamOneOne(
                            operand_var.clone(),
                            box LExpr::AppOneCont(
                                box LExpr::Var(operator_var.clone()),
                                box LExpr::Var(operand_var.clone()),
                                box new_cont,
                            ),
                        ),
                        ctx,
                    ),
                ),
                ctx,
            )
        }
        LExpr::AppOneCont(..) | LExpr::Lit(..) => expr,
        LExpr::Lam(..) | LExpr::App(..) | LExpr::LamOne(..) => unreachable!("These shouldn't exist here"),
    }
}

/// Apply the cps transformation
pub fn cps_transform<'a>(expr: LExpr<'a>, ctx: &mut TransformContext) -> LExpr<'a> {
    match expr {
        LExpr::LamOneOne(arg, box expr) => {
            let cont_var: Cow<'a, str> = ctx.gen_ident("cont_var");
            let cont_var_exp = LExpr::Var(cont_var.clone());
            let rv_var: Cow<'a, str> = ctx.gen_ident("rv_var");
            let rv_var_exp = LExpr::Var(rv_var.clone());
            let cont = LExpr::LamOneOne(
                rv_var.clone(),
                box LExpr::AppOne(box cont_var_exp.clone(),  box rv_var_exp),
            );
            LExpr::LamOneOneCont(
                arg,
                cont_var.clone(),
                box cps_transform_cont(expr, cont.clone(), ctx),
            )
        }
        x => x,
    }
}
