use std::{
    collections::HashMap,
    borrow::Cow,
};
use cdsl::{CStmt, CExpr, CDecl, CType};
use nodes::{LExpr, Env, LExEnv, LamType};
// use transform::TransformContext;

// Process: every lambda body defines new bindings
// Each binding gets associated with a unique index


#[derive(Debug)]
pub struct EnvCtx<'a> {
    var_index: usize,
    lam_index: usize,
    pub lam_map: Vec<(usize, Env<'a>)>,
}

impl<'a> EnvCtx<'a> {
    pub fn new() -> Self {
        EnvCtx {
            var_index: 0,
            lam_index: 0,
            lam_map: Vec::new(),
        }
    }

    pub fn gen_var_index(&mut self) -> usize {
        let index = self.var_index;
        self.var_index += 1;
        index
    }

    pub fn gen_env_index(&mut self) -> usize {
        let index = self.lam_index;
        self.lam_index += 1;
        index
    }

    /// Insert an environment list into the table of environments
    pub fn add_lam_map(&mut self, env: Env<'a>) -> usize {
        let index = self.gen_env_index();
        self.lam_map.push((index, env));
        index
    }
}

/// Resolve variables into explicit environments, aswell as producing a map of environments in use
fn resolve_env_internal<'a>(node: LExpr<'a>, env: &Env<'a>, ctx: &mut EnvCtx<'a>) -> LExEnv<'a> {
    match node {
        LExpr::Var(name) => LExEnv::Var {
            name: name.clone(),
            global: !env.get(&name).is_some(),
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
            let arg_index = (arg.clone(), ctx.gen_var_index());

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
            let arg_index = (arg.clone(), ctx.gen_var_index());
            let cont_index = (cont.clone(), ctx.gen_var_index());

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
        _ => unreachable!("Node of type {:?} should not exist here.", node),
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
            (LamRef { id, lam_type: LamType::OneArg }, extracted_lambdas)
        },
        LamCont { arg, cont, box expr, env, id } => {
            let (inner_expr, mut extracted_lambdas) = extract_lambdas(expr);
            let new = LamCont { arg, cont,
                                expr: box inner_expr,
                                env, id };
            extracted_lambdas.insert(id, new);
            (LamRef { id, lam_type: LamType::TwoArg }, extracted_lambdas)
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


pub struct CodegenCtx {
    unique_var_id: usize,
}


impl CodegenCtx {
    pub fn new() -> Self {
        CodegenCtx {
            unique_var_id: 0
        }
    }

    fn gen_var_id(&mut self) -> usize {
        let var_id = self.unique_var_id;
        self.unique_var_id += 1;
        var_id
    }

    fn gen_var<'a>(&mut self) -> Cow<'a, str> {
        Cow::Owned(format!("unique_var_{}", self.gen_var_id()))
    }
}


pub fn lambda_codegen<'a>(lams: &Vec<LExEnv<'a>>) -> Vec<CDecl<'a>> {
    use self::LExEnv::*;

    lams.iter().map(
        |lam| match lam {
            Lam { arg, box expr, env: _, id } => {
                let mut supporting_stmts = Vec::new();
                let mut ctx = CodegenCtx::new();

                let name = format!("lambda_{}", id);

                let args = vec![(arg.clone(), CType::Ptr(box CType::Struct(Cow::Borrowed("object"))))];
                let main_expr = CStmt::Expr(codegen(&expr, &mut ctx, &mut supporting_stmts));;

                let mut body = Vec::new();
                body.extend(supporting_stmts);
                body.push(main_expr);

                CDecl::Fun {
                    name: Cow::Owned(name),
                    typ: CType::Void,
                    args: args,
                    body: body,
                }
            },
            LamCont { arg, cont, box expr, env: _, id } => {
                let mut supporting_stmts = Vec::new();
                let mut ctx = CodegenCtx::new();

                let name = format!("lambda_{}", id);

                let args = vec![
                    (arg.clone(),  CType::Ptr(box CType::Struct(Cow::Borrowed("object")))),
                    (cont.clone(), CType::Ptr(box CType::Struct(Cow::Borrowed("object")))),
                ];

                let main_expr = CStmt::Expr(codegen(&expr, &mut ctx, &mut supporting_stmts));;

                let mut body = Vec::new();
                body.extend(supporting_stmts);
                body.push(main_expr);

                CDecl::Fun {
                    name: Cow::Owned(name),
                    typ: CType::Void,
                    args: args,
                    body: body,
                }
            },
            _ => unreachable!("Should not exist here"),
        }
    ).collect()
}


/// Generates C code for an expression
pub fn codegen<'a>(expr: &LExEnv<'a>, ctx: &mut CodegenCtx, supporting_stmts: &mut Vec<CStmt<'a>>) -> CExpr<'a> {
    use self::LExEnv::*;

    match expr {
        LamRef { id, lam_type } => {
            let lam_name = Cow::Owned(format!("lambda_{}", id));

            let result_var = ctx.gen_var();

            let lambda_generate = CStmt::Decl(
                CDecl::Var {
                    name: result_var.clone(),
                    typ: CType::Struct(Cow::Borrowed("closure")),
                    init: Some(CExpr::FunCallOp {
                        expr: box CExpr::Ident(lam_type.ctor_func()),
                        ands: vec![CExpr::LitInt(*id), CExpr::Ident(lam_name), CExpr::Ident(Cow::Borrowed("env"))],
                    }),
                }
            );

            supporting_stmts.push(lambda_generate);

            CExpr::Cast {
                ex: box CExpr::PreUnOp {
                    op: Cow::Borrowed("&"),
                    ex: box CExpr::Ident(result_var),
                },
                typ: CType::Ptr(box CType::Other(Cow::Borrowed("object"))),
            }
        },
        Var { name, global: true, .. } =>
            gen_global_lookup(name.clone()),
        Var { name, global: false, env } => {
            println!("var name: {}, env: {:#?}", name, env);
            gen_local_lookup(env.get(name).expect("Variable should exist in environment"))
        },
        App1 { cont, rand, .. } => {
            let cont_compiled = codegen(cont, ctx, supporting_stmts);
            let rand_compiled = codegen(rand, ctx, supporting_stmts);

            CExpr::FunCallOp {
                expr: box CExpr::Ident(Cow::Borrowed("call_closure_one")),
                ands: vec![cont_compiled, rand_compiled],
            }
        },
        App2 { rator, rand, cont, .. } => {
            let rator_compiled = codegen(rator, ctx, supporting_stmts);
            let rand_compiled = codegen(rand, ctx, supporting_stmts);
            let cont_compiled = codegen(cont, ctx, supporting_stmts);

            CExpr::FunCallOp {
                expr: box CExpr::Ident(Cow::Borrowed("call_closure_two")),
                ands: vec![rator_compiled, rand_compiled, cont_compiled],
            }
        },
        _ => unreachable!("Should not exist here"),
    }
}


fn gen_global_lookup<'a>(name: Cow<'a, str>) -> CExpr<'a> {
    // TODO: me
    CExpr::LitStr(Cow::Owned(format!("!global_lookup_for_{}_here!", name)))
}


fn gen_local_lookup<'a>(id: usize) -> CExpr<'a> {
    CExpr::FunCallOp {
        expr: box CExpr::Ident(Cow::Borrowed("env_get")),
        ands: vec![CExpr::LitInt(id)],
    }
}


fn gen_env_table_elem<'a, 'b>(id: usize, env: &'a Env<'a>) -> CExpr<'b> {
    let mut args = vec![CExpr::LitInt(id)];
    args.extend(env.0.values().map(|&v| CExpr::LitInt(v)));

    CExpr::MacroCall {
        name: Cow::Borrowed("ENV_ENTRY"),
        args: args,
    }
}


fn gen_builtin_envs<'a>(ctx: &mut EnvCtx<'a>) -> Vec<(String, usize, Env<'a>)> {

    fn make_builtin_binop<'a>(ctx: &mut EnvCtx<'a>, name: &str) -> Vec<(String, usize, Env<'a>)> {
        let mut first_env = HashMap::new();
        first_env.insert(Cow::Owned(format!("{}_param_1", name)), ctx.gen_var_index());

        let mut second_env = HashMap::new();
        second_env.insert(Cow::Owned(format!("{}_param_1", name)), ctx.gen_var_index());
        second_env.insert(Cow::Owned(format!("{}_param_2", name)), ctx.gen_var_index());

        vec![
            (format!("{}_env_1", name), ctx.gen_env_index(), Env(first_env)),
            (format!("{}_env_2", name), ctx.gen_env_index(), Env(second_env)),
        ]
    }

    let builtins = vec![
        make_builtin_binop(ctx, "int_obj_add"),
        make_builtin_binop(ctx, "int_obj_sub"),
        make_builtin_binop(ctx, "int_obj_mul"),
        make_builtin_binop(ctx, "int_obj_div"),
    ].into_iter().flatten().collect();

    builtins
}


/// generate the environment ids, stuff
pub fn gen_env_ids<'a>(ctx: &mut EnvCtx<'a>, program_envs: Vec<(usize, Env<'a>)>) -> Vec<CDecl<'a>> {
    let builtin_envs = gen_builtin_envs(ctx);

    let builtin_var_ids: Vec<_> = builtin_envs.iter()
                                              .flat_map(|(_, _, e)| e.0.clone())
                                              .collect();

    let builtin_env_ids: Vec<_> = builtin_envs.iter()
                                              .map(|(name, id, _)| (name.to_owned(), *id))
                                              .collect();

    let mut env_table_entries = Vec::new();

    env_table_entries.extend(builtin_envs.iter().map(|(_, id, env)| gen_env_table_elem(*id, env)));
    env_table_entries.extend(program_envs.iter().map(|(id, env)| gen_env_table_elem(*id, env)));

    let global_env_table_decl = CDecl::Var {
        name: Cow::Borrowed("global_env_table"),
        typ: CType::Arr(box CType::Struct(Cow::Borrowed("env_table_entry")), None),
        init: Some(CExpr::InitList(env_table_entries)),
    };

    let builtin_var_ids_decl: Vec<_> = builtin_var_ids.iter()
                                              .map(|(name, id)| CDecl::Var {
                                                  name: name.clone(),
                                                  typ: CType::Other(Cow::Borrowed("size_t")),
                                                  init: Some(CExpr::LitInt(*id)),
                                              })
                                              .collect();
    let builtin_env_ids_decl: Vec<_> = builtin_env_ids.into_iter()
                                              .map(|(name, id)| CDecl::Var {
                                                  name: Cow::Owned(name),
                                                  typ: CType::Other(Cow::Borrowed("size_t")),
                                                  init: Some(CExpr::LitInt(id)),
                                              })
                                              .collect();

    let mut results = Vec::new();
    results.push(global_env_table_decl);
    results.extend(builtin_var_ids_decl);
    results.extend(builtin_env_ids_decl);
    results
}
