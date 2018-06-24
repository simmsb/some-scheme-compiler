#ifndef SOMESCHEME_BUILTIN_H
#define SOMESCHEME_BUILTIN_H

#include "base.h"

#define DEFINE_BUILTIN_VAR(NAME) extern size_t NAME
#define DEFINE_BUILTIN_ENV(NAME) extern size_t NAME

#define DEFINE_TWO_ARG_FROM_BUILTIN(NAME)                                      \
    void NAME##_func_1(struct object *, struct object *, struct env_elem *);   \
    void NAME##_func_2(struct object *, struct object *, struct env_elem *)

#define MAKE_TWO_ARG_FROM_BUILTIN(NAME, TYPE, LHS_ARG, RHS_ARG, LHS_ENV,       \
                                  RHS_ENV)                                     \
    void NAME##_func_1(struct object *rand, struct object *cont,               \
                       struct env_elem *env) {                                 \
        ADD_ENV(LHS_ARG, rand, &env);                                          \
                                                                               \
        struct closure func_2_clos =                                           \
            object_closure_two_new(LHS_ENV, NAME##_func_2, env);               \
                                                                               \
        call_closure_one(cont, (struct object *)&func_2_clos);                 \
    }                                                                          \
    void NAME##_func_2(struct object *rand, struct object *cont,               \
                       struct env_elem *env) {                                 \
        ADD_ENV(RHS_ARG, rand, &env);                                          \
                                                                               \
        struct object *lhs = env_get(LHS_ARG, env);                            \
                                                                               \
        TYPE result = (NAME)(lhs, rand);                                       \
                                                                               \
        call_closure_one(cont, (struct object *)&result);                      \
    }

#define DEFINE_BINOP(NAME)                                                     \
    DEFINE_BUILTIN_VAR(NAME##_param_1);                                        \
    DEFINE_BUILTIN_VAR(NAME##_param_2);                                        \
    DEFINE_BUILTIN_ENV(NAME##_env_1);                                          \
    DEFINE_BUILTIN_ENV(NAME##_env_2);                                          \
    DEFINE_TWO_ARG_FROM_BUILTIN(NAME)

// builtin operations

struct int_obj int_obj_add(struct object *, struct object *);

struct int_obj int_obj_sub(struct object *, struct object *);

struct int_obj int_obj_mul(struct object *, struct object *);

struct int_obj int_obj_div(struct object *, struct object *);

DEFINE_BINOP(int_obj_add);
DEFINE_BINOP(int_obj_sub);
DEFINE_BINOP(int_obj_mul);
DEFINE_BINOP(int_obj_div);

#endif // SOMESCHEME_BUILTIN_H
