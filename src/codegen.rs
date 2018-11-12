use std::{
    collections::HashMap,
    borrow::Cow,
};
use cdsl::{CStmt, CExpr, CDecl, CType};
use nodes::{LExpr, Env, LExEnv, LamType, ExprLit};
// use itertools::Itertools;
// use transform::TransformContext;

// Process: every lambda body defines new bindings
// Each binding gets associated with a unique index


#[derive(Debug, Default)]
pub struct EnvCtx<'a> {
    var_index: usize,
    lam_index: usize,
    pub lam_map: Vec<(usize, Env<'a>)>,
}

impl<'a> EnvCtx<'a> {
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
            name,
            env: env.clone(),
        },
        LExpr::BuiltinIdent(name, arity) => LExEnv::BuiltinIdent(name, arity),
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
                arg,
                expr: box resolve_env_internal(expr, &new_env, ctx),
                env: new_env,
                id,
            }
        },
        LExpr::LamOneOneCont(arg, cont, box expr) => {
            let arg_index = (arg.clone(), ctx.gen_var_index());
            let cont_index = (cont.clone(), ctx.gen_var_index());

            let new_env = Env::new(env, vec![arg_index, cont_index]);
            let id = ctx.add_lam_map(new_env.clone());

            LExEnv::LamCont {
                arg,
                cont,
                expr: box resolve_env_internal(expr, &new_env, ctx),
                env: new_env,
                id,
            }
        },
        LExpr::Lit(x) => LExEnv::Lit(x),
        _ => unreachable!("Node of type {:?} should not exist here.", node),
    }
}

pub fn resolve_env(node: LExpr) -> (LExEnv, EnvCtx) {
    let mut ctx = EnvCtx::default();
    let primary_env = Env::default();

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


#[derive(Default)]
pub struct CodegenCtx {
    unique_var_id: usize,
}


impl CodegenCtx {
    fn gen_var_id(&mut self) -> usize {
        let var_id = self.unique_var_id;
        self.unique_var_id += 1;
        var_id
    }

    fn gen_var<'a>(&mut self) -> Cow<'a, str> {
        Cow::from(format!("unique_var_{}", self.gen_var_id()))
    }
}


fn env_set_codegen<'a>(arg: &'a str, env: &Env<'a>) -> CStmt<'a> {
    let index = env.get(arg).expect("env has argument");

    CStmt::Expr(
        CExpr::MacroCall {
            name: "ADD_ENV".into(),
            args: vec![
                CExpr::LitUInt(index),
                CExpr::Ident(arg.into()),
                CExpr::PreUnOp {
                    op: "&".into(),
                    ex: box CExpr::Ident("env".into())
                }
            ]
        }
    )
}


pub fn lambda_codegen<'a>(lams: &'a [LExEnv<'a>]) -> Vec<CDecl<'a>> {
    use self::LExEnv::*;

    lams.iter().map(
        |lam| match lam {
            Lam { arg, box expr, env, id } => {
                let mut supporting_stmts = Vec::new();
                let mut ctx = CodegenCtx::default();

                let name = format!("lambda_{}", id);

                let args = vec![
                    (arg.clone(), CType::Ptr(box CType::Struct(Cow::from("object")))),
                    (Cow::from("env"), CType::Ptr(box CType::Struct(Cow::from("env_elem")))),
                ];

                supporting_stmts.push(env_set_codegen(&arg, &env));

                let main_expr = CStmt::Expr(codegen(&expr, &mut ctx, &mut supporting_stmts));;

                let mut body = Vec::new();
                body.extend(supporting_stmts);
                body.push(main_expr);

                CDecl::Fun {
                    name: Cow::from(name),
                    typ: CType::Static(box CType::Void),
                    args,
                    body,
                }
            },
            LamCont { arg, cont, box expr, env, id } => {
                let mut supporting_stmts = Vec::new();
                let mut ctx = CodegenCtx::default();

                let name = format!("lambda_{}", id);

                let args = vec![
                    (arg.clone(),  CType::Ptr(box CType::Struct(Cow::from("object")))),
                    (cont.clone(), CType::Ptr(box CType::Struct(Cow::from("object")))),
                    (Cow::from("env"), CType::Ptr(box CType::Struct(Cow::from("env_elem")))),
                ];

                supporting_stmts.push(env_set_codegen(&arg, &env));
                supporting_stmts.push(env_set_codegen(&cont, &env));

                let main_expr = CStmt::Expr(codegen(&expr, &mut ctx, &mut supporting_stmts));;

                let mut body = Vec::new();
                body.extend(supporting_stmts);
                body.push(main_expr);

                CDecl::Fun {
                    name: Cow::from(name),
                    typ: CType::Static(box CType::Void),
                    args,
                    body,
                }
            },
            _ => unreachable!("Should not exist here"),
        }
    ).collect()
}


pub fn lambda_proto_codegen<'a>(lams: &[LExEnv<'a>]) -> Vec<CDecl<'a>> {
    use self::LExEnv::*;

    lams.iter().map(
        |lam| match lam {
            Lam { id, .. } => {
                let name = format!("lambda_{}", id);

                let args = vec![
                    CType::Ptr(box CType::Struct(Cow::from("object"))),
                    CType::Ptr(box CType::Struct(Cow::from("env_elem"))),
                ];

                CDecl::FunProto {
                    name: Cow::from(name),
                    typ: CType::Static(box CType::Void),
                    args,
                }
            },
            LamCont { id, .. } => {
                let name = format!("lambda_{}", id);

                let args = vec![
                    CType::Ptr(box CType::Struct(Cow::from("object"))),
                    CType::Ptr(box CType::Struct(Cow::from("object"))),
                    CType::Ptr(box CType::Struct(Cow::from("env_elem"))),
                ];

                CDecl::FunProto {
                    name: Cow::from(name),
                    typ: CType::Static(box CType::Void),
                    args,
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
            let lam_name = Cow::from(format!("lambda_{}", id));

            let result_var = ctx.gen_var();

            let lambda_generate = CStmt::Decl(
                CDecl::Var {
                    name: result_var.clone(),
                    typ: CType::Struct(Cow::from("closure")),
                    init: Some(CExpr::FunCallOp {
                        expr: box CExpr::Ident(lam_type.ctor_func()),
                        ands: vec![CExpr::LitUInt(*id), CExpr::Ident(lam_name), CExpr::Ident(Cow::from("env"))],
                    }),
                }
            );

            supporting_stmts.push(lambda_generate);

            CExpr::Cast {
                ex: box CExpr::PreUnOp {
                    op: Cow::from("&"),
                    ex: box CExpr::Ident(result_var),
                },
                typ: CType::Ptr(box CType::Struct(Cow::from("object"))),
            }
        },
        Var { name, env } => {
            gen_local_lookup(env.get(name).expect(&format!("Variable {} should exist in environment", name)))
        },
        BuiltinIdent(name, arity) => {
            let result_var = ctx.gen_var();

            let ctor_fun = match arity {
                LamType::OneArg => "object_closure_one_new",
                LamType::TwoArg => "object_closure_two_new",
            };

            let closure = CStmt::Decl(
                CDecl::Var {
                    name: result_var.clone(),
                    typ: CType::Struct(Cow::from("closure")),
                    init: Some(CExpr::FunCallOp {
                        expr: box CExpr::Ident(Cow::from(ctor_fun)),
                        ands: vec![
                            CExpr::Ident(Cow::from(format!("{}_env", name))),
                            CExpr::Ident(Cow::from(format!("{}_func", name))),
                            CExpr::Ident(Cow::from("env"))
                        ],
                    }),
                }
            );

            supporting_stmts.push(closure);

            CExpr::Cast {
                ex: box CExpr::PreUnOp {
                    op: Cow::from("&"),
                    ex: box CExpr::Ident(result_var),
                },
                typ: CType::Ptr(box CType::Struct(Cow::from("object"))),
            }
        },
        App1 { cont, rand, .. } => {
            let cont_compiled = codegen(cont, ctx, supporting_stmts);
            let rand_compiled = codegen(rand, ctx, supporting_stmts);

            CExpr::FunCallOp {
                expr: box CExpr::Ident(Cow::from("call_closure_one")),
                ands: vec![cont_compiled, rand_compiled],
            }
        },
        App2 { rator, rand, cont, .. } => {
            let rator_compiled = codegen(rator, ctx, supporting_stmts);
            let rand_compiled = codegen(rand, ctx, supporting_stmts);
            let cont_compiled = codegen(cont, ctx, supporting_stmts);

            CExpr::FunCallOp {
                expr: box CExpr::Ident(Cow::from("call_closure_two")),
                ands: vec![rator_compiled, rand_compiled, cont_compiled],
            }
        },
        Lit(x) => {
            let temp_var = ctx.gen_var();

            let macro_call = match x {
                ExprLit::Void => CExpr::MacroCall {
                    name: "OBJECT_VOID_OBJ_NEW".into(),
                    args: vec![CExpr::Ident(temp_var.clone())],
                },
                ExprLit::NumLit(x) => CExpr::MacroCall {
                    name: "OBJECT_INT_OBJ_NEW".into(),
                    args: vec![CExpr::LitIInt(*x as isize), CExpr::Ident(temp_var.clone())],
                },
                ExprLit::StringLit(x) => CExpr::MacroCall {
                    name: "OBJECT_STRING_OBJ_NEW".into(),
                    args: vec![CExpr::LitStr(x.clone()), CExpr::Ident(temp_var.clone())],
                },
            };

            supporting_stmts.push(CStmt::Expr(macro_call));

            CExpr::Ident(temp_var)
        },
        _ => unreachable!("Should not exist here"),
    }
}

fn gen_local_lookup<'a>(id: usize) -> CExpr<'a> {
    CExpr::FunCallOp {
        expr: box CExpr::Ident(Cow::from("env_get")),
        ands: vec![CExpr::LitUInt(id), CExpr::Ident(Cow::from("env"))],
    }
}


fn gen_env_table_elem<'a, 'b>(id: usize, env: &'a Env<'a>) -> CExpr<'b> {
    let mut args = vec![CExpr::LitUInt(id)];
    args.extend(env.0.values().map(|&v| CExpr::LitUInt(v)));

    CExpr::MacroCall {
        name: Cow::from("ENV_ENTRY"),
        args,
    }
}


#[derive(Constructor)]
struct CompleteEnv<'a> {
    name: Cow<'a, str>,
    id: usize,
    env: Env<'a>,
}


#[derive(Constructor)]
struct CompleteVar<'a> {
    name: Cow<'a, str>,
    id: usize,
}


// TODO: document what we do here
fn gen_builtin_envs<'a>(ctx: &mut EnvCtx<'a>) -> (Vec<CompleteEnv<'a>>, Vec<CompleteVar<'a>>) {

    fn make_builtin_binop<'a>(ctx: &mut EnvCtx<'a>, name: &str) -> (Vec<CompleteEnv<'a>>, Vec<CompleteVar<'a>>) {
        let (first_var_name, first_var_id)   = (Cow::from(format!("{}_param", name)), ctx.gen_var_index());
        let (second_var_name, second_var_id) = (Cow::from(format!("{}_param_2", name)), ctx.gen_var_index());

        let mut first_env = HashMap::new();
        first_env.insert(first_var_name.clone(), first_var_id);

        let mut second_env = HashMap::new();
        second_env.insert(first_var_name.clone(), first_var_id);
        second_env.insert(second_var_name.clone(), second_var_id);

        let envs = vec![
            CompleteEnv::new(Cow::from(format!("{}_env", name)), ctx.gen_env_index(), Env(first_env)),
            CompleteEnv::new(Cow::from(format!("{}_env_2", name)), ctx.gen_env_index(), Env(second_env)),
        ];

        let vars = vec![
            CompleteVar::new(first_var_name, first_var_id),
            CompleteVar::new(second_var_name, second_var_id),
        ];

        (envs, vars)
    }

    fn make_builtin_unop<'a>(ctx: &mut EnvCtx<'a>, name: &str) -> (CompleteEnv<'a>, CompleteVar<'a>) {
        let (var_name, var_id) = (Cow::from(format!("{}_param", name)), ctx.gen_var_index());

        let mut env = HashMap::new();
        env.insert(var_name.clone(), var_id);

        let env = CompleteEnv::new(Cow::from(format!("{}_env", name)), ctx.gen_env_index(), Env(env));
        let var = CompleteVar::new(var_name, var_id);

        (env, var)
    }

    let (builtin_binops_envs, builtin_binops_vars): (Vec<_>, Vec<_>) = vec![
        make_builtin_binop(ctx, "object_int_obj_add"),
        make_builtin_binop(ctx, "object_int_obj_sub"),
        make_builtin_binop(ctx, "object_int_obj_mul"),
        make_builtin_binop(ctx, "object_int_obj_div"),
    ].into_iter().unzip();

    let builtin_binops_envs = builtin_binops_envs
        .into_iter()
        .flat_map(|x| x); // use flat map for now (until itertools is sorted out)

    let builtin_binops_vars = builtin_binops_vars
        .into_iter()
        .flat_map(|x| x);

    let (builtin_unops_envs, builtin_unops_vars): (Vec<_>, Vec<_>) = vec![
        make_builtin_unop(ctx, "halt_func"),
        make_builtin_unop(ctx, "to_string_func"),
        make_builtin_unop(ctx, "println_func"),
    ].into_iter().unzip();

    let envs = builtin_binops_envs.chain(builtin_unops_envs).collect();
    let vars = builtin_binops_vars.chain(builtin_unops_vars).collect();

    (envs, vars)
}


/// generate the environment ids, stuff
pub fn gen_env_ids<'a>(ctx: &mut EnvCtx<'a>, program_envs: &[(usize, Env<'a>)]) -> Vec<CDecl<'a>> {
    let (builtin_envs, builtin_vars) = gen_builtin_envs(ctx);

    let mut env_table_entries = Vec::new();

    env_table_entries.extend(builtin_envs.iter().map(|CompleteEnv { id, env, .. }| gen_env_table_elem(*id, env)));
    env_table_entries.extend(program_envs.iter().map(|(id, env)| gen_env_table_elem(*id, env)));

    let global_env_table_decl = CDecl::Var {
        name: Cow::from("global_env_table"),
        typ: CType::Arr(box CType::Struct(Cow::from("env_table_entry")), None),
        init: Some(CExpr::InitList(env_table_entries)),
    };

    let builtin_var_ids_decl: Vec<_> = builtin_vars
        .iter()
        .map(|CompleteVar { name, id }| CDecl::Var {
            name: name.clone(),
            typ: CType::Other(Cow::from("size_t")),
            init: Some(CExpr::LitUInt(*id)),
        })
        .collect();

    let builtin_env_ids_decl: Vec<_> = builtin_envs
        .iter()
        .map(|CompleteEnv { name, id, .. }| CDecl::Var {
            name: name.clone(),
            typ: CType::Other(Cow::from("size_t")),
            init: Some(CExpr::LitUInt(*id)),
        })
        .collect();

    let mut results = Vec::new();
    results.push(global_env_table_decl);
    results.extend(builtin_var_ids_decl);
    results.extend(builtin_env_ids_decl);
    results
}
