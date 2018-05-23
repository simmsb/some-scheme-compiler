use std::borrow::Cow;

use parse::LExpr;

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

    pub fn gen_var(&mut self) -> String {
        let var = format!("$anon_var_{}", self.genvar_count);
        self.genvar_count += 1;
        var
    }
}


/// Transform multiple parameter lambdas into nested single parmeters.
/// (lambda (a b c) ...)
/// becomes
/// (lambda (a)
///   (lambda (b)
///     (lambda (c)
///       ...)))
pub fn expand_lam<'a>(expr: LExpr<'a>, mut ctx: &mut TransformContext) -> LExpr<'a> {
    match expr {
        LExpr::Lam(args, body) => {
            let body: Vec<_> = body.into_iter().map(|x| expand_lam(x, &mut ctx)).collect();
            match args.len() {
                0 => LExpr::LamNone(body),
                _ => {
                    let mut iter = args.into_iter().rev();

                    let first = LExpr::LamOne(iter.next().unwrap(), body);

                    iter.fold(
                        first, |acc, arg| LExpr::LamOne(arg, vec![acc])
                    )
                }
            }
        },
        LExpr::App(box operator, operands) => {
            let operator = expand_lam(operator, &mut ctx);
            let operands = operands.into_iter().map(|o| expand_lam(o, &mut ctx)).collect();

            LExpr::App(box operator, operands)
        },
        x => x,
    }
}


/// Transform calls with multiple parameters into nested calls each applying single parameters
/// (f a b c)
/// becomes
/// ((((f) a) b) c)
pub fn expand_app<'a>(expr: LExpr<'a>, mut ctx: &mut TransformContext) -> LExpr<'a> {
    match expr {
        LExpr::Lam(..) => panic!("expand_app should be used after expand_lam"),
        LExpr::App(box operator, operands) => {
            let num_operands = operands.len();
            let operator = expand_app(operator, &mut ctx);
            let operands: Vec<_> = operands.into_iter().map(|o| expand_app(o, &mut ctx)).collect();
            match num_operands {
                0 => LExpr::AppNone(box operator),
                _ => {
                    // let mut iter = operands.into_iter();

                    let mut operands = operands.into_iter();

                    let first = LExpr::AppOne(box operator, box operands.next().unwrap());

                    operands.fold(
                        first, |acc, arg| LExpr::AppOne(box acc, box arg)
                    )
                }
            }
        },
        LExpr::LamNone(body)     => LExpr::LamNone(body.into_iter().map(|b| expand_app(b, &mut ctx)).collect()),
        LExpr::LamOne(arg, body) => LExpr::LamOne(arg, body.into_iter().map(|b| expand_app(b, &mut ctx)).collect()),
        x => x,
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
pub fn expand_lam_body<'a>(expr: LExpr<'a>, mut ctx: &mut TransformContext) -> LExpr<'a> {
    match expr {
        LExpr::Lam(..) | LExpr::App(..) => panic!("expand_lam_body should be used after expand_lam and expand_app"),
        LExpr::LamNone(body) => {
            let num_body = body.len();
            let body: Vec<_> = body.into_iter()
                                   .rev()
                                   .map(|b| expand_lam_body(b, &mut ctx))
                                   .collect();
            let inner = match num_body {
                0 => LExpr::LamNoneNone, // (lambda ()) ; wat
                _ => {
                    // get the last expression, as this won't be placed in a (... x) wrapper
                    let mut body = body.into_iter();
                    let first = body.next().unwrap();

                    body.fold(
                        first, |acc, operand| LExpr::AppOne(
                            box LExpr::LamOneOne(Cow::Owned(ctx.gen_var()), box acc),
                            box operand
                        )
                    )
                }
            };
            LExpr::LamNoneOne(box inner)
        },
        LExpr::LamOne(arg, body) => {
            let num_body = body.len();
            let body: Vec<_> = body.into_iter()
                                   .rev()
                                   .map(|b| expand_lam_body(b, &mut ctx))
                                   .collect();
            let inner = match num_body {
                0 => LExpr::LamNoneNone, // (lambda ()) ; wat
                _ => {
                    // get the last expression, as this won't be placed in a (... x) wrapper
                    let mut body = body.into_iter();
                    let first = body.next().unwrap();

                    body.fold(
                        first, |acc, arg| LExpr::AppOne(
                            box LExpr::LamOneOne(Cow::Owned(ctx.gen_var()), box acc),
                            box arg
                        )
                    )
                }
            };
            LExpr::LamOneOne(arg, box inner)
        },
        LExpr::AppNone(box operator) => LExpr::AppNone(box expand_lam_body(operator, &mut ctx)),
        LExpr::AppOne(box operator, box operand) => LExpr::AppOne(box expand_lam_body(operator, &mut ctx),
                                                                  box expand_lam_body(operand, &mut ctx)),
        x => x,
    }
}



// Cps conversion
//
// Rules:
//
//
// pub fn
