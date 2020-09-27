use std::collections::HashMap;
use std::rc::Rc;

use moniker::FreeVar;
use moniker::Ignore;

use crate::cdsl::CDecl;
use crate::cdsl::CExpr;
use crate::cdsl::CStmt;
use crate::cdsl::CType;
use crate::lifted_expr::LExpr;
use crate::lifted_expr::LiftedLambda;
use crate::literals::Literal;

pub struct CodegenCtx<'a> {
    unique_var_id: usize,
    protos: Vec<CDecl<'static>>,
    declarations: Vec<CDecl<'static>>,
    lambdas: &'a HashMap<usize, LiftedLambda>,
}

impl<'a> CodegenCtx<'a> {
    pub fn new(lambdas: &'a HashMap<usize, LiftedLambda>) -> Self {
        Self {
            unique_var_id: 0,
            protos: Vec::new(),
            declarations: Vec::new(),
            lambdas,
        }
    }

    fn gen_var_id(&mut self) -> usize {
        let var_id = self.unique_var_id;
        self.unique_var_id += 1;
        var_id
    }

    fn gen_var(&mut self) -> String {
        format!("var_{}", self.gen_var_id())
    }

    fn add_proto(&mut self, proto: CDecl<'static>) {
        self.protos.push(proto);
    }

    fn add_decl(&mut self, decl: CDecl<'static>) {
        self.declarations.push(decl);
    }
}

fn name_for_free_var(var: &FreeVar<String>) -> String {
    let name = var
        .pretty_name
        .as_deref()
        .unwrap_or("anon")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>();

    format!("v_{}_{}", name, var.unique_id)
}

fn object_type() -> CType<'static> {
    CType::Ptr(Rc::new(CType::Struct("obj".into())))
}

impl LiftedLambda {
    fn env_struct(&self) -> CDecl<'static> {
        let members = self
            .freevars
            .iter()
            .map(|v| (name_for_free_var(v).into(), object_type()))
            .collect();

        CDecl::Struct {
            name: format!("env_{}", self.id).into(),
            members,
        }
    }

    fn construct_env_code(&self, dest: &str) -> CStmt<'static> {
        CStmt::Expr(CExpr::MacroCall {
            name: "OBJECT_ENV_OBJ_NEW".into(),
            args: vec![
                Rc::new(CExpr::Ident(dest.to_owned().into())),
                Rc::new(CType::Struct(format!("env_{}", self.id).into())),
            ],
        })
    }

    fn make_env_code(
        &self,
        src_env: &Rc<CExpr<'static>>,
        ctx: &mut CodegenCtx,
        supporting_stmts: &mut Vec<Rc<CStmt<'static>>>,
    ) -> Rc<CExpr<'static>> {
        let var_name = ctx.gen_var();

        supporting_stmts.push(Rc::new(self.construct_env_code(&var_name)));

        let env_expr = Rc::new(CExpr::Ident(var_name.into()));
        let env_access = Rc::new(self.generate_env_cast(env_expr.clone()));

        let mut vars_to_copy = self.freevars.clone();
        for param in &self.params {
            vars_to_copy.remove(param);
        }

        for var in &vars_to_copy {
            supporting_stmts.push(Rc::new(CStmt::Expr(CExpr::BinOp {
                op: "=".into(),
                left: Rc::new(CExpr::Arrow {
                    expr: env_access.clone(),
                    attr: name_for_free_var(var).into(),
                }),
                right: Rc::new(CExpr::Arrow {
                    expr: src_env.clone(),
                    attr: name_for_free_var(var).into(),
                }),
            })));
        }

        env_expr
    }

    fn generate_closure(
        &self,
        current_env: &Rc<CExpr<'static>>,
        ctx: &mut CodegenCtx,
        supporting_stmts: &mut Vec<Rc<CStmt<'static>>>,
    ) -> CExpr<'static> {
        let env_expr = self.make_env_code(current_env, ctx, supporting_stmts);

        let init_name = match self.params.len() {
            1 => "OBJECT_CLOSURE_ONE_NEW",
            2 => "OBJECT_CLOSURE_TWO_NEW",
            n => panic!("closure was not one or two parameters, was: {}", n),
        };

        let var_name = ctx.gen_var();

        let init_stmt = CStmt::Expr(CExpr::MacroCall {
            name: init_name.into(),
            args: vec![
                Rc::new(CExpr::Ident(var_name.to_owned().into())),
                Rc::new(CExpr::Ident(format!("lambda_{}", self.id).into())),
                env_expr,
            ],
        });

        supporting_stmts.push(Rc::new(init_stmt));

        CExpr::Ident(var_name.into())
    }

    fn generate_env_cast(&self, in_expr: Rc<CExpr<'static>>) -> CExpr<'static> {
        CExpr::Cast {
            ex: Rc::new(CExpr::PreUnOp {
                op: "&".into(),
                ex: Rc::new(CExpr::Arrow {
                    expr: in_expr,
                    attr: "env".into(),
                }),
            }),
            typ: self.generate_env_ptr_typ(),
        }
    }

    fn generate_env_ptr_typ(&self) -> CType<'static> {
        CType::Ptr(Rc::new(CType::Struct(format!("env_{}", self.id).into())))
    }

    fn generate_func(&self, ctx: &mut CodegenCtx) {
        let params = self
            .params
            .iter()
            .map(|p| (p.clone(), ctx.gen_var()))
            .collect::<Vec<_>>();

        let env_obj_s = Rc::new(CType::Struct("env_obj".into()));
        let obj_s = Rc::new(CType::Struct("obj".into()));

        let (mut with_names, mut types_only): (Vec<_>, Vec<_>) = params
            .iter()
            .map(|(_, n)| {
                (
                    (n.to_owned().into(), CType::Ptr(obj_s.clone())),
                    CType::Ptr(obj_s.clone()),
                )
            })
            .unzip();

        with_names.push(("env_in".into(), CType::Ptr(env_obj_s.clone())));
        types_only.push(CType::Ptr(env_obj_s.clone()));

        let proto = CDecl::FunProto {
            name: format!("lambda_{}", self.id).into(),
            typ: CType::Void,
            args: types_only,
            noreturn: true,
        };

        ctx.add_proto(proto);

        let env_move_stmt = Rc::new(CStmt::Decl(CDecl::Var {
            name: "env".into(),
            typ: self.generate_env_ptr_typ(),
            init: Some(self.generate_env_cast(Rc::new(CExpr::Ident("env_in".into())))),
        }));

        let env_expr = Rc::new(CExpr::Ident("env".into()));

        let mut stmts: Vec<Rc<CStmt<'static>>> = vec![env_move_stmt];

        for (dest_var, in_var) in &params {
            if !self.freevars.contains(dest_var) {
                // if a parameter isn't used in the body, we discard it instead of actually using it
                continue;
            }

            let tmp_name = ctx.gen_var();
            let tmp_var = Rc::new(CExpr::Ident(tmp_name.into()));

            stmts.push(Rc::new(CStmt::Expr(CExpr::MacroCall {
                name: "OBJECT_CELL_OBJ_NEW".into(),
                args: vec![
                    tmp_var.clone(),
                    Rc::new(CExpr::Ident(in_var.to_owned().into())),
                ],
            })));

            stmts.push(Rc::new(CStmt::Expr(CExpr::BinOp {
                op: "=".into(),
                left: Rc::new(CExpr::Arrow {
                    expr: env_expr.clone(),
                    attr: name_for_free_var(dest_var).into(),
                }),
                right: tmp_var,
            })));
        }

        let final_expr = do_codegen_internal(&self.body, ctx, &mut stmts);
        stmts.push(Rc::new(CStmt::Expr(final_expr)));

        stmts.push(Rc::new(CStmt::Expr(CExpr::MacroCall {
            name: "__builtin_unreachable".into(),
            args: vec![],
        })));

        let fun = CDecl::Fun {
            name: format!("lambda_{}", self.id).into(),
            typ: CType::Void,
            args: with_names,
            body: stmts,
        };

        ctx.add_decl(fun);
    }
}

pub fn do_codegen(
    e: LExpr,
    lambdas: &HashMap<usize, LiftedLambda>,
) -> (
    Vec<Rc<CStmt<'static>>>,
    Vec<CDecl<'static>>,
    Vec<CDecl<'static>>,
) {
    let mut ctx = CodegenCtx::new(lambdas);
    let mut stmts = Vec::new();

    for lambda in lambdas.values() {
        ctx.add_proto(lambda.env_struct());
        lambda.generate_func(&mut ctx);
    }

    let final_expr = do_codegen_internal(&e, &mut ctx, &mut stmts);
    stmts.push(Rc::new(CStmt::Expr(final_expr)));

    (stmts, ctx.protos, ctx.declarations)
}

fn builtin_ident_codegen(
    ident: &str,
    ctx: &mut CodegenCtx,
    supporting_stmts: &mut Vec<Rc<CStmt<'static>>>,
) -> CExpr<'static> {
    let (num_params, runtime_name) = match ident {
        "tostring" => (2, "to_string_k"), // these are two-param because they take the cont param
        "display" => (2, "display_k"),
        "exit" => (1, "exit_k"),
        "+" => (2, "add_k"),
        "-" => (2, "sub_k"),
        "*" => (2, "mul_k"),
        "/" => (2, "div_k"),
        "^" => (2, "xor_k"),
        "cons" => (2, "cons_k"),
        "cons?" => (2, "is_cons_k"),
        "null?" => (2, "is_null_k"),
        "car" => (2, "car_k"),
        "cdr" => (2, "cdr_k"),
        "string-concat" => (2, "string_concat_k"),
        _ => panic!("unknown builtin: {}", ident),
    };

    let init_name = match num_params {
        1 => "OBJECT_CLOSURE_ONE_NEW",
        2 => "OBJECT_CLOSURE_TWO_NEW",
        n => panic!("closure was not one or two parameters, was: {}", n),
    };

    let var_name = ctx.gen_var();

    let init_stmt = CStmt::Expr(CExpr::MacroCall {
        name: init_name.into(),
        args: vec![
            Rc::new(CExpr::Ident(var_name.to_owned().into())),
            Rc::new(CExpr::Ident(runtime_name.into())),
            Rc::new(CExpr::Ident("NULL".into())),
        ],
    });

    supporting_stmts.push(Rc::new(init_stmt));

    CExpr::Ident(var_name.into())
}

fn do_codegen_internal(
    e: &LExpr,
    ctx: &mut CodegenCtx,
    supporting_stmts: &mut Vec<Rc<CStmt<'static>>>,
) -> CExpr<'static> {
    match e {
        LExpr::Var(v) => {
            let resolved_name = match v {
                moniker::Var::Free(f) => name_for_free_var(f),
                moniker::Var::Bound(_) => panic!("bound var: {:?}", v),
            };
            CExpr::Arrow {
                expr: Rc::new(CExpr::Cast {
                    typ: CType::Ptr(Rc::new(CType::Struct("cell_obj".into()))),
                    ex: Rc::new(CExpr::Arrow {
                        expr: Rc::new(CExpr::Ident("env".into())),
                        attr: resolved_name.into(),
                    }),
                }),
                attr: "val".into(),
            }
        }
        LExpr::Lit(Ignore(l)) => {
            let (ctor_name, expr) = match l {
                Literal::String(s) => ("OBJECT_STRING_OBJ_NEW", CExpr::LitStr(s.to_owned().into())),
                Literal::Int(i) => ("OBJECT_INT_OBJ_NEW", CExpr::LitIInt(*i as isize)),
                Literal::Float(_f) => panic!("not yet"),
                Literal::Void => return CExpr::Ident("NULL".into()),
            };

            let dest = ctx.gen_var();

            let init_stmt = CStmt::Expr(CExpr::MacroCall {
                name: ctor_name.to_owned().into(),
                args: vec![Rc::new(CExpr::Ident(dest.to_owned().into())), Rc::new(expr)],
            });

            supporting_stmts.push(Rc::new(init_stmt));

            CExpr::Ident(dest.into())
        }
        LExpr::BuiltinIdent(Ignore(i)) => builtin_ident_codegen(i.as_ref(), ctx, supporting_stmts),
        LExpr::SetThen(v, e, c) => {
            let e_expr = do_codegen_internal(e, ctx, supporting_stmts);
            let resolved_name = match v {
                moniker::Var::Free(f) => name_for_free_var(f),
                moniker::Var::Bound(_) => panic!("bound var: {:?}", v),
            };
            let var_exp = CExpr::Arrow {
                expr: Rc::new(CExpr::Cast {
                    typ: CType::Ptr(Rc::new(CType::Struct("cell_obj".into()))),
                    ex: Rc::new(CExpr::Arrow {
                        expr: Rc::new(CExpr::Ident("env".into())),
                        attr: resolved_name.into(),
                    }),
                }),
                attr: "val".into(),
            };

            supporting_stmts.push(Rc::new(CStmt::Expr(CExpr::BinOp {
                op: "=".into(),
                left: Rc::new(var_exp),
                right: Rc::new(e_expr),
            })));

            do_codegen_internal(c, ctx, supporting_stmts)
        }
        LExpr::Lifted(Ignore(id)) => {
            let lambda = ctx.lambdas.get(id).unwrap();
            let env_expr = Rc::new(CExpr::Ident("env".into()));
            lambda.generate_closure(&env_expr, ctx, supporting_stmts)
        }
        LExpr::If(c, ift, iff) => {
            let mut ift_stmts = Vec::new();
            let ift = do_codegen_internal(ift, ctx, &mut ift_stmts);
            ift_stmts.push(Rc::new(CStmt::Expr(ift)));

            let mut iff_stmts = Vec::new();
            let iff = do_codegen_internal(iff, ctx, &mut iff_stmts);
            iff_stmts.push(Rc::new(CStmt::Expr(iff)));

            let stmt = CStmt::If {
                cond: CExpr::MacroCall {
                    name: "obj_is_truthy".into(),
                    args: vec![Rc::new(do_codegen_internal(c, ctx, supporting_stmts))],
                },
                ift: Rc::new(CStmt::Block(ift_stmts)),
                iff: Rc::new(CStmt::Block(iff_stmts)),
            };

            supporting_stmts.push(Rc::new(stmt));

            CExpr::LitIInt(0)
        }
        LExpr::CallOne(c, a) => CExpr::MacroCall {
            name: "call_closure_one".into(),
            args: vec![
                Rc::new(do_codegen_internal(c, ctx, supporting_stmts)),
                Rc::new(do_codegen_internal(a, ctx, supporting_stmts)),
            ],
        },
        LExpr::CallTwo(c, a, k) => CExpr::MacroCall {
            name: "call_closure_two".into(),
            args: vec![
                Rc::new(do_codegen_internal(c, ctx, supporting_stmts)),
                Rc::new(do_codegen_internal(a, ctx, supporting_stmts)),
                Rc::new(do_codegen_internal(k, ctx, supporting_stmts)),
            ],
        },
    }
}
