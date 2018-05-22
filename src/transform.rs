use parse::LExpr;

// compiler transformation stage


/// Transform multiple parameter lambdas into nested single parmeters.
/// (lambda (a b c) ...)
/// becomes
/// (lambda (a)
///   (lambda (b)
///     (lambda (c)
///       ...)))
pub fn expand_lam<'a>(expr: LExpr<'a>) -> LExpr<'a> {

    match expr {
        LExpr::Lam(args, body) => {
            let body: Vec<_> = body.into_iter().map(expand_lam).collect();
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
        LExpr::App(box rator, args) => {
            let operator = expand_lam(rator);
            let args = args.into_iter().map(expand_lam).collect();

            LExpr::App(box operator, args)
        },
        x => x,
    }
}
