#ifndef SOMESCHEME_BUILTIN_H
#define SOMESCHEME_BUILTIN_H

#include "base.h"

#define DEFINE_ZERO_ARG_FROM_BUILTIN(NAME)                                     \
  void NAME##_k(struct obj *, struct env_obj *) __attribute__((noreturn))

#define MAKE_ZERO_ARG_FROM_BUILTIN(NAME, INNER, TYPE)                          \
  void NAME##_k(struct obj *k, struct env_obj *env) {                          \
    TYPE result = (INNER)();                                                   \
                                                                               \
    call_closure_one(k, (struct obj *)&result);                                \
                                                                               \
    __builtin_unreachable();                                                   \
  }

#define DEFINE_ONE_ARG_FROM_BUILTIN(NAME)                                      \
  void NAME##_k(struct obj *, struct obj *, struct env_obj *)

#define MAKE_ONE_ARG_FROM_BUILTIN(NAME, INNER, TYPE)                           \
  void NAME##_k(struct obj *v, struct obj *k, struct env_obj *env) {           \
    TYPE result = (INNER)(v);                                                  \
                                                                               \
    call_closure_one(k, (struct obj *)&result);                                \
                                                                               \
    __builtin_unreachable();                                                   \
  }

#define DEFINE_TWO_ARG_FROM_BUILTIN(NAME)                                      \
  void NAME##_k(struct obj *, struct obj *, struct env_obj *)                  \
      __attribute__((noreturn));                                               \
  void NAME##_k_2(struct obj *, struct obj *, struct env_obj *)                \
      __attribute__((noreturn))

struct unary_env {
  struct obj *val;
};

#define MAKE_TWO_ARG_FROM_BUILTIN(NAME, INNER, TYPE)                           \
  void NAME##_k(struct obj *v, struct obj *k, struct env_obj *env) {           \
    OBJECT_ENV_OBJ_NEW(tmp_env, struct unary_env);                             \
    tmp_env->env[0] = v;                                                       \
    struct closure_obj func_2_clos =                                           \
        object_closure_two_new(NAME##_k_2, tmp_env);                           \
                                                                               \
    call_closure_one(k, (struct obj *)&func_2_clos);                           \
                                                                               \
    __builtin_unreachable();                                                   \
  }                                                                            \
  void NAME##_k_2(struct obj *v, struct obj *k, struct env_obj *env) {         \
                                                                               \
    TYPE result = (INNER)(env->env[0], v);                                     \
                                                                               \
    call_closure_one(k, (struct obj *)&result);                                \
                                                                               \
    __builtin_unreachable();                                                   \
  }

// builtin operations
DEFINE_TWO_ARG_FROM_BUILTIN(add);
DEFINE_TWO_ARG_FROM_BUILTIN(sub);
DEFINE_TWO_ARG_FROM_BUILTIN(mul);
DEFINE_TWO_ARG_FROM_BUILTIN(div);
DEFINE_TWO_ARG_FROM_BUILTIN(xor);

DEFINE_TWO_ARG_FROM_BUILTIN(cons);

DEFINE_TWO_ARG_FROM_BUILTIN(string_concat);

DEFINE_ZERO_ARG_FROM_BUILTIN(exit);

DEFINE_ONE_ARG_FROM_BUILTIN(to_string);
DEFINE_ONE_ARG_FROM_BUILTIN(println);

DEFINE_ONE_ARG_FROM_BUILTIN(car);
DEFINE_ONE_ARG_FROM_BUILTIN(cdr);

_Bool obj_is_truthy(struct obj *);

#endif // SOMESCHEME_BUILTIN_H
