#ifndef SOMESCHEME_BUILTIN_H
#define SOMESCHEME_BUILTIN_H

#include "base.h"

#define DEFINE_BUILTIN_VAR(NAME) extern size_t NAME
#define DEFINE_BUILTIN_ENV(NAME) extern size_t NAME

#define DEFINE_ONE_ARG_FROM_BUILTIN(NAME)                                      \
  void NAME##_func(struct object *, struct object *, struct env_elem *)

#define MAKE_ONE_ARG_FROM_BUILTIN(NAME, TYPE, ARG, ENV)                        \
  void NAME##_func(struct object *rand, struct object *cont,                   \
                   struct env_elem *env) {                                     \
    ADD_ENV(ARG, rand, &env);                                                  \
                                                                               \
    struct object *lhs = env_get(ARG, env);                                    \
                                                                               \
    TYPE result = (NAME)(lhs);                                                 \
                                                                               \
    call_closure_one(cont, (struct object *)&result);                          \
  }

#define DEFINE_TWO_ARG_FROM_BUILTIN(NAME)                                      \
  void NAME##_func(struct object *, struct object *, struct env_elem *);       \
  void NAME##_func_2(struct object *, struct object *, struct env_elem *)

#define DEFINE_UNOP(NAME)                                                      \
  DEFINE_BUILTIN_VAR(NAME##_param);                                            \
  DEFINE_BUILTIN_ENV(NAME##_env);                                              \
  DEFINE_ONE_ARG_FROM_BUILTIN(NAME)

#define MAKE_CONSTRUCTOR_BUILTIN(NAME, TYPE)                                   \
  void NAME##_ctor(TYPE inp, struct object *cont, struct env_elem *env) {      \
    TYPE result = (NAME)(inp);                                                 \
                                                                               \
    call_closure_one(cont, (struct object *)&result);                          \
  }

#define MAKE_TWO_ARG_FROM_BUILTIN(NAME, TYPE, LHS_ARG, RHS_ARG, LHS_ENV,       \
                                  RHS_ENV)                                     \
  void NAME##_func(struct object *rand, struct object *cont,                   \
                   struct env_elem *env) {                                     \
    ADD_ENV(LHS_ARG, rand, &env);                                              \
                                                                               \
    struct closure func_2_clos =                                               \
        object_closure_two_new(LHS_ENV, NAME##_func_2, env);                   \
                                                                               \
    call_closure_one(cont, (struct object *)&func_2_clos);                     \
  }                                                                            \
  void NAME##_func_2(struct object *rand, struct object *cont,                 \
                     struct env_elem *env) {                                   \
    ADD_ENV(RHS_ARG, rand, &env);                                              \
                                                                               \
    struct object *lhs = env_get(LHS_ARG, env);                                \
                                                                               \
    TYPE result = (NAME)(lhs, rand);                                           \
                                                                               \
    call_closure_one(cont, (struct object *)&result);                          \
  }

#define DEFINE_BINOP(NAME)                                                     \
  DEFINE_BUILTIN_VAR(NAME##_param);                                            \
  DEFINE_BUILTIN_VAR(NAME##_param_2);                                          \
  DEFINE_BUILTIN_ENV(NAME##_env);                                              \
  DEFINE_BUILTIN_ENV(NAME##_env_2);                                            \
  DEFINE_TWO_ARG_FROM_BUILTIN(NAME)

// builtin operations

struct int_obj object_int_obj_add(struct object *, struct object *);

struct int_obj object_int_obj_sub(struct object *, struct object *);

struct int_obj object_int_obj_mul(struct object *, struct object *);

struct int_obj object_int_obj_div(struct object *, struct object *);

DEFINE_BINOP(object_int_obj_add);
DEFINE_BINOP(object_int_obj_sub);
DEFINE_BINOP(object_int_obj_mul);
DEFINE_BINOP(object_int_obj_div);

DEFINE_BUILTIN_VAR(halt_func_param);
DEFINE_BUILTIN_ENV(halt_func_env);
void halt_func_func(struct object *, struct env_elem *);

DEFINE_BUILTIN_VAR(to_string_func_param);
DEFINE_BUILTIN_ENV(to_string_func_env);
void to_string_func_func(struct object *, struct object *, struct env_elem *);

DEFINE_BUILTIN_VAR(println_func_param);
DEFINE_BUILTIN_ENV(println_func_env);
void println_func_func(struct object *, struct object *, struct env_elem *);

#endif // SOMESCHEME_BUILTIN_H
