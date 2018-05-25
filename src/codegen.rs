use nodes::{LExpr, Env, LExEnv};
use transform::TransformContext;


/// Resolve variables into explicit environments
fn resolve_variables<'a>(node: LExpr<'a>) -> LExEnv<'a> {
    // TODO: me
    unreachable!()
}


/// Recurses through each node and extracts each lambda such that no nested lambdas exist
/// Returns the transformed root node and all extracted lambdas
fn extract_lambdas<'a>(node: LExpr<'a>, ctx: &mut TransformContext) -> (LExpr<'a>, Vec<LExpr<'a>>) {
    use LExpr::*;

    // TODO: me
    unreachable!();

    let mut lifted = Vec::new();

    match node {
        LamOneOneCont(arg, box expr, box cont) => {
            let
        }
    }

}
