use std::borrow::Cow;

use nodes::LExpr;

// compiler transformation stage


pub struct TransformContext {
    genvar_count: u64,
}


impl TransformContext {
    pub fn new() -> Self {
        TransformContext {
            genvar_count: 0,
        }
    }

    pub fn gen_ident(&mut self) -> String {
        let var = format!("$anon_var_{}", self.genvar_count);
        self.genvar_count += 1;
        var
    }

    pub fn gen_var<'a>(&mut self) -> LExpr<'a> {
        LExpr::Var(Cow::Owned(self.gen_ident()))
    }

    pub fn gen_cont<'a>(&mut self) -> LExpr<'a> {
        let var = format!("$cont_var_{}", self.genvar_count);
        self.genvar_count += 1;
        LExpr::Var(Cow::Owned(var))
    }
}


/// Transform multiple parameter lambdas into nested single parmeters.
/// (lambda (a b c) ...)
/// becomes
/// (lambda (a)
///   (lambda (b)
///     (lambda (c)
///       ...)))
/// Transform calls with multiple parameters into nested calls each applying single parameters
/// (f a b c)
/// becomes
/// ((((f) a) b) c)
pub fn expand_lam_app<'a>(expr: LExpr<'a>, ctx: &mut TransformContext) -> LExpr<'a> {
    match expr {
        LExpr::Lam(args, body) => {
            let body: Vec<_> = body.into_iter().map(|x| expand_lam_app(x, ctx)).collect();
            match args.len() {
                0 => LExpr::LamNone(body),
                _ => {
                    let mut iter = args.into_iter().rev();

                    let first = LExpr::LamOne(iter.next().unwrap(), body);

                    iter.fold(
                        first, |acc, arg| LExpr::LamOneOne(arg, box acc)
                    )
                }
            }
        },
        LExpr::App(box operator, operands) => {
            let operator = expand_lam_app(operator, ctx);
            let operands: Vec<_> = operands.into_iter().map(|o| expand_lam_app(o, ctx)).collect();
            let num_operands = operands.len();
            match num_operands {
                0 => LExpr::AppNone(box operator),
                _ => {
                    let mut operands = operands.into_iter();

                    let first = LExpr::AppOne(box operator, box operands.next().unwrap());

                    operands.fold(
                        first, |acc, arg| LExpr::AppOne(box acc, box arg)
                    )
                }
            }
        },
        LExpr::Var(arg) => LExpr::Var(arg),
        _ => panic!("Shouldn't be touching this yet"),
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
        LamNone(body) => {
            let num_body = body.len();
            let body: Vec<_> = body.into_iter()
                                   .rev()
                                   .map(|b| expand_lam_body(b, ctx))
                                   .collect();
            let inner = match num_body {
                0 => LamNoneNone, // (lambda ()) ; wat
                _ => {
                    // get the last expression, as this won't be placed in a (... x) wrapper
                    let mut body = body.into_iter();
                    let first = body.next().unwrap();

                    body.fold(
                        first, |acc, operand| AppOne(
                            box LamOneOne(Cow::Owned(ctx.gen_ident()), box acc),
                            box operand
                        )
                    )
                }
            };
            LamNoneOne(box inner)
        },
        LamOne(arg, body) => {
            let num_body = body.len();
            let body: Vec<_> = body.into_iter()
                                   .rev()
                                   .map(|b| expand_lam_body(b, ctx))
                                   .collect();
            let inner = match num_body {
                0 => LamNoneNone, // (lambda ()) ; wat
                _ => {
                    // get the last expression, as this won't be placed in a (... x) wrapper
                    let mut body = body.into_iter();
                    let first = body.next().unwrap();

                    body.fold(
                        first, |acc, arg| AppOne(
                            box LamOneOne(Cow::Owned(ctx.gen_ident()), box acc),
                            box arg
                        )
                    )
                }
            };
            LamOneOne(arg, box inner)
        },
        AppNone(box operator) => AppNone(box expand_lam_body(operator, ctx)),
        AppOne(box operator, box operand) => AppOne(box expand_lam_body(operator, ctx),
                                                    box expand_lam_body(operand, ctx)),
        x => x,
    }
}


pub fn cps_transform_cont<'a>(expr: LExpr<'a>, cont: LExpr<'a>, ctx: &mut TransformContext) -> LExpr<'a> {
    match expr {
        LExpr::Var(..) |
        LExpr::LamNoneOne(..) |
        LExpr::LamNoneNone |
        LExpr::LamOneOne(..) |
        LExpr::LamOneNone(..) |
        LExpr::LamNoneNoneCont(..) |
        LExpr::LamOneOneCont(..) |
        LExpr::LamOneNoneCont(..) =>
            LExpr::AppOne(box cont, box cps_transform(expr, ctx)),
        LExpr::AppNone(box operator) => {
            let o: Cow<'a, str> = Cow::Owned(ctx.gen_ident());
            let o_var = LExpr::Var(o.clone());

            let new_cont = LExpr::LamOneOne(
                o.clone(),
                box LExpr::AppOne(box o_var, box cont)
            );

            cps_transform_cont(operator, new_cont, ctx)
        },
        LExpr::AppOne(box operator, box operand) => {
            let ator: Cow<'a, str> = Cow::Owned(ctx.gen_ident());
            let rand: Cow<'a, str> = Cow::Owned(ctx.gen_ident());

            let new_cont = LExpr::LamOneOne(
                ator.clone(),
                box cps_transform_cont(
                    operand,
                    LExpr::LamOneOne(
                        rand.clone(),
                        box LExpr::AppOneCont(
                            box LExpr::Var(ator.clone()),
                            box LExpr::Var(rand.clone()),
                            box cont
                        ),
                    ),
                    ctx
                )
            );

            cps_transform_cont(operator, new_cont, ctx)
        },
        LExpr::AppOneCont(..) | LExpr::LamNoneOneCont(..) => expr,
        LExpr::Lam(..) | LExpr::App(..) | LExpr::LamNone(..) | LExpr::LamOne(..) =>
            panic!("These shouldn't exist here"),
    }
}


pub fn cps_transform<'a>(expr: LExpr<'a>, ctx: &mut TransformContext) -> LExpr<'a> {
    match expr {
        LExpr::LamNoneOne(box expr) => {
            let cont = ctx.gen_cont();
            LExpr::LamNoneOneCont(
                box cps_transform_cont(expr, cont.clone(), ctx),
                box cont.clone()
            )
        },
        LExpr::LamNoneNone =>
            LExpr::LamNoneNoneCont(box ctx.gen_cont()),
        LExpr::LamOneOne(arg, box expr) => {
            let cont = ctx.gen_cont();
            LExpr::LamOneOneCont(
                arg,
                box cps_transform_cont(expr, cont.clone(), ctx),
                box cont.clone()
            )
        },
        LExpr::LamOneNone(arg) =>
            LExpr::LamOneNoneCont(arg, box ctx.gen_cont()),
        x => x,
    }
}
