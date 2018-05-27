use nodes::{LExpr, Env, LExEnv, EnvCtx};
// use transform::TransformContext;


/// Resolve variables into explicit environments
fn resolve_env_internal<'a>(node: LExpr<'a>, env: &Env<'a>, ctx: &mut EnvCtx) -> LExEnv<'a> {
    match node {
        LExpr::Var(name) => LExEnv::Var {
            name: name.clone(),
            global: env.get(&name).is_some(),
            env: env.clone(),
        },
        LExpr::AppOne(box operator, box operand) => {
            let cont    = resolve_env_internal(operator, env, ctx);
            let operand = resolve_env_internal(operand,  env, ctx);

            LExEnv::App1 {
                cont: box cont,
                rand: box operand,
                env: env.clone(),
            }
        }
        LExpr::AppOneCont(box operator, box operand, box cont) => {
            let operator = resolve_env_internal(operator, env, ctx);
            let operand  = resolve_env_internal(operand,  env, ctx);
            let cont     = resolve_env_internal(cont,     env, ctx);

            LExEnv::App2 {
                rator: box operator,
                rand: box operand,
                cont: box cont,
                env: env.clone(),
            }
        },
        LExpr::LamOneOne(arg, box expr) => {
            let arg_index = (arg.clone(), ctx.get_index());

            let new_env = Env::new(env, vec![arg_index]);

            LExEnv::Lam {
                arg: arg,
                expr: box resolve_env_internal(expr, &new_env, ctx),
                env: new_env,
            }
        },
        LExpr::LamOneOneCont(arg, cont, box expr) => {
            let arg_index = (arg.clone(), ctx.get_index());
            let cont_index = (cont.clone(), ctx.get_index());

            let new_env = Env::new(env, vec![arg_index, cont_index]);

            LExEnv::LamCont {
                arg: arg,
                cont: cont,
                expr: box resolve_env_internal(expr, &new_env, ctx),
                env: new_env,
            }
        },
        _ => panic!("Node of type {:?} should not exist here.", node),
    }
}

pub fn resolve_env<'a>(node: LExpr<'a>) -> LExEnv<'a> {
    let mut ctx = EnvCtx::new();
    let primary_env = Env::empty();

    let resolved = resolve_env_internal(node, &primary_env, &mut ctx);

    resolved
}


// /// Recurses through each node and extracts each lambda such that no nested lambdas exist
// /// Returns the transformed root node and all extracted lambdas
// fn extract_lambdas<'a>(node: LExpr<'a>, ctx: &mut TransformContext) -> (LExpr<'a>, Vec<LExpr<'a>>) {
//     use LExpr::*;

//     // TODO: me
//     unreachable!();

//     let mut lifted = Vec::new();

//     match node {
//         LamOneOneCont(arg, box expr, box cont) => {
//             let
//         }
//     }

// }
