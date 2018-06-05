use std::collections::HashMap;
use nodes::{LExpr, Env, LExEnv};
// use transform::TransformContext;

// Process: every lambda body defines new bindings
// Each binding gets associated with a unique index


#[derive(Debug)]
pub struct EnvCtx<'a> {
    var_index: usize,
    lam_map: Vec<Env<'a>>,
}

impl<'a> EnvCtx<'a> {
    pub fn new() -> Self {
        EnvCtx {
            var_index: 0,
            lam_map: Vec::new(),
        }
    }

    pub fn get_var_index(&mut self) -> usize {
        let index = self.var_index;
        self.var_index += 1;
        index
    }

    /// Insert an environment list into the table of environments
    pub fn add_lam_map(&mut self, env: Env<'a>) -> usize {
        let index = self.lam_map.len();
        self.lam_map.push(env);
        index
    }
}

/// Resolve variables into explicit environments, aswell as producing a map of environments in use
fn resolve_env_internal<'a>(node: LExpr<'a>, env: &Env<'a>, ctx: &mut EnvCtx<'a>) -> LExEnv<'a> {
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
            let arg_index = (arg.clone(), ctx.get_var_index());

            let new_env = Env::new(env, vec![arg_index]);
            let id = ctx.add_lam_map(new_env.clone());

            LExEnv::Lam {
                arg: arg,
                expr: box resolve_env_internal(expr, &new_env, ctx),
                env: new_env,
                id: id,
            }
        },
        LExpr::LamOneOneCont(arg, cont, box expr) => {
            let arg_index = (arg.clone(), ctx.get_var_index());
            let cont_index = (cont.clone(), ctx.get_var_index());

            let new_env = Env::new(env, vec![arg_index, cont_index]);
            let id = ctx.add_lam_map(new_env.clone());

            LExEnv::LamCont {
                arg: arg,
                cont: cont,
                expr: box resolve_env_internal(expr, &new_env, ctx),
                env: new_env,
                id: id,
            }
        },
        _ => panic!("Node of type {:?} should not exist here.", node),
    }
}

pub fn resolve_env<'a>(node: LExpr<'a>) -> (LExEnv<'a>, EnvCtx<'a>) {
    let mut ctx = EnvCtx::new();
    let primary_env = Env::empty();

    let resolved = resolve_env_internal(node, &primary_env, &mut ctx);

    (resolved, ctx)
}


/// Given an expression, extract all lambdas, replacing lambdas with references
pub fn extract_lambdas<'a>(node: LExEnv<'a>) -> (LExEnv<'a>, HashMap<usize, LExEnv<'a>>) {
    use self::LExEnv::*;

    match node {
        Lam { arg, box expr, env, id } => {
            let (inner_expr, mut extracted_lambdas) = extract_lambdas(expr);
            let new = Lam { arg, expr: box inner_expr, env, id };
            extracted_lambdas.insert(id, new);
            (LamRef {id}, extracted_lambdas)
        },
        LamCont { arg, cont, box expr, env, id } => {
            let (inner_expr, mut extracted_lambdas) = extract_lambdas(expr);
            let new = LamCont { arg, cont,
                                expr: box inner_expr,
                                env, id };
            extracted_lambdas.insert(id, new);
            (LamRef {id}, extracted_lambdas)
        },
        App1 { box cont, box rand, env } => {
            let (new_cont, cont_lambdas) = extract_lambdas(cont);
            let (new_rand, rand_lambdas) = extract_lambdas(rand);

            let mut lambdas = cont_lambdas;
            lambdas.extend(rand_lambdas);

            let new = App1 { cont: box new_cont,
                             rand: box new_rand, env };
            (new, lambdas)
        },
        App2 { box rator, box rand, box cont, env } => {
            let (new_rator, rator_lambdas) = extract_lambdas(rator);
            let (new_rand, rand_lambdas)   = extract_lambdas(rand);
            let (new_cont, cont_lambdas)   = extract_lambdas(cont);

            let mut lambdas = rator_lambdas;
            lambdas.extend(rand_lambdas);
            lambdas.extend(cont_lambdas);

            let new = App2 { rator: box new_rator,
                             rand: box new_rand,
                             cont: box new_cont,
                             env };
            (new, lambdas)
        },
        x => (x, HashMap::new()),
    }
}
