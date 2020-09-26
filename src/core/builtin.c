#include "builtin.h"
#include "base.h"
#include "common.h"

#define MAKE_INT_BINOP(NAME, OP)                                               \
  struct int_obj object_int_obj_##NAME(struct obj *lhs, struct obj *rhs) {     \
    if (lhs->tag != OBJ_INT)                                                   \
      RUNTIME_ERROR("Left operand to binary add not of integer type");         \
    if (rhs->tag != OBJ_INT)                                                   \
      RUNTIME_ERROR("Right operand to binary add not of integer type");        \
                                                                               \
    struct int_obj *lhs_int = (struct int_obj *)lhs;                           \
    struct int_obj *rhs_int = (struct int_obj *)rhs;                           \
                                                                               \
    return object_int_obj_new(lhs_int->val OP rhs_int->val);                   \
  } MAKE_TWO_ARG_FROM_BUILTIN(NAME, object_int_obj_##NAME, struct int_obj)

MAKE_INT_BINOP(add, +);
MAKE_INT_BINOP(sub, -);
MAKE_INT_BINOP(mul, *);
MAKE_INT_BINOP(div, /);
MAKE_INT_BINOP(xor, ^);

MAKE_TWO_ARG_FROM_BUILTIN(cons, object_cons_obj_new, struct cons_obj);

int exit_inner() { exit(0); }

MAKE_ZERO_ARG_FROM_BUILTIN(exit, exit_inner, int);

char *obj_to_string_internal(struct obj *val) {
  char *res;

  if (!val) {
    ALLOC_SPRINTF(res, "()");
  }

  switch (val->tag) {
  case OBJ_CONS:
    ALLOC_SPRINTF(res, "cons");
    break;
  case OBJ_CLOSURE:
    ALLOC_SPRINTF(res, "closure|%p", (void *)((struct closure_obj *)val)->fn_1);
    break;
  case OBJ_INT:
    ALLOC_SPRINTF(res, "%ld", ((struct int_obj *)val)->val);
    break;
  case OBJ_STR:
    ALLOC_SPRINTF(res, "%s", ((struct string_obj *)val)->buf);
    break;
  case OBJ_CELL:
    return obj_to_string_internal(((struct cell_obj *)val)->val);
  default:
    RUNTIME_ERROR("Unexpected object tag to to_string: %d", val->tag);
  }

  return res;
}

void to_string_k(struct obj *v, struct obj *k, struct env_obj *env) {
  char *res = obj_to_string_internal(v);

  OBJECT_STRING_OBJ_NEW(result_str, res);

  free(res);

  call_closure_one(k, result_str);

  __builtin_unreachable();
}

void println_k(struct obj *v, struct obj *k, struct env_obj *env) {
  char *res = obj_to_string_internal(v);

  printf("%s\n", res);

  free(res);

  call_closure_one(k, NULL);

  __builtin_unreachable();
}

_Bool obj_is_truthy(struct obj *obj) {
  switch (obj->tag) {
  case OBJ_INT:
    return ((struct int_obj *)obj)->val != 0;
  case OBJ_STR:
    return ((struct string_obj *)obj)->len != 0;
  default:
    return true;
  }
}

void car_k(struct obj *cons, struct obj *k, struct env_obj *env) {
  struct obj *car = ((struct cons_obj *)cons)->car;

  call_closure_one(k, car);

  __builtin_unreachable();
}

void cdr_k(struct obj *cons, struct obj *k, struct env_obj *env) {
  struct obj *cdr = ((struct cons_obj *)cons)->cdr;

  call_closure_one(k, cdr);

  __builtin_unreachable();
}

void string_concat_k(struct obj *v, struct obj *k, struct env_obj *env) {
  OBJECT_ENV_OBJ_NEW(tmp_env, struct unary_env);
  tmp_env->env[0] = v;
  struct closure_obj func_2_clos =
      object_closure_two_new(string_concat_k_2, tmp_env);

  call_closure_one(k, (struct obj *)&func_2_clos);

  __builtin_unreachable();
}

static char *convert_to_str(struct obj *v) {
  char *res;

  switch (v->tag) {
  case OBJ_INT:
    ALLOC_SPRINTF(res, "%c", (int)((struct int_obj *)v)->val);
    break;
  case OBJ_STR:
    ALLOC_SPRINTF(res, "%s", ((struct string_obj *)v)->buf);
    break;
  case OBJ_CELL:
    res = convert_to_str(((struct cell_obj *)v)->val);
    break;
  default:
    RUNTIME_ERROR("Unexpected object tag to convert_to_str: %d", v->tag);
  }

  return res;
}

void string_concat_k_2(struct obj *v, struct obj *k, struct env_obj *env) {
  char *lhs = convert_to_str(env->env[0]);
  char *rhs = convert_to_str(v);

  char *res;
  ALLOC_SPRINTF(res, "%s%s", lhs, rhs);

  free(lhs);
  free(rhs);

  OBJECT_STRING_OBJ_NEW(result_str, res);

  free(res);

  call_closure_one(k, result_str);

  __builtin_unreachable();
}
