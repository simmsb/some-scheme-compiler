#include "builtin.h"
#include "base.h"
#include "common.h"

struct int_obj object_int_obj_add(struct obj *lhs, struct obj *rhs) {
  if (lhs->tag != OBJ_INT)
    RUNTIME_ERROR("Left operand to binary add not of integer type");
  if (rhs->tag != OBJ_INT)
    RUNTIME_ERROR("Right operand to binary add not of integer type");

  struct int_obj *lhs_int = (struct int_obj *)lhs;
  struct int_obj *rhs_int = (struct int_obj *)rhs;

  return object_int_obj_new(lhs_int->val + rhs_int->val);
}

struct int_obj object_int_obj_sub(struct obj *lhs, struct obj *rhs) {
  if (lhs->tag != OBJ_INT)
    RUNTIME_ERROR("Left operand to binary sub not of integer type");
  if (rhs->tag != OBJ_INT)
    RUNTIME_ERROR("Right operand to binary sub not of integer type");

  struct int_obj *lhs_int = (struct int_obj *)lhs;
  struct int_obj *rhs_int = (struct int_obj *)rhs;

  return object_int_obj_new(lhs_int->val - rhs_int->val);
}

struct int_obj object_int_obj_mul(struct obj *lhs, struct obj *rhs) {
  if (lhs->tag != OBJ_INT)
    RUNTIME_ERROR("Left operand to binary mul not of integer type");
  if (rhs->tag != OBJ_INT)
    RUNTIME_ERROR("Right operand to binary mul not of integer type");

  struct int_obj *lhs_int = (struct int_obj *)lhs;
  struct int_obj *rhs_int = (struct int_obj *)rhs;

  return object_int_obj_new(lhs_int->val * rhs_int->val);
}

struct int_obj object_int_obj_div(struct obj *lhs, struct obj *rhs) {
  if (lhs->tag != OBJ_INT)
    RUNTIME_ERROR("Left operand to binary div not of integer type");
  if (rhs->tag != OBJ_INT)
    RUNTIME_ERROR("Right operand to binary div not of integer type");

  struct int_obj *lhs_int = (struct int_obj *)lhs;
  struct int_obj *rhs_int = (struct int_obj *)rhs;

  return object_int_obj_new(lhs_int->val / rhs_int->val);
}

MAKE_TWO_ARG_FROM_BUILTIN(add, object_int_obj_add, struct int_obj);
MAKE_TWO_ARG_FROM_BUILTIN(sub, object_int_obj_sub, struct int_obj);
MAKE_TWO_ARG_FROM_BUILTIN(mul, object_int_obj_mul, struct int_obj);
MAKE_TWO_ARG_FROM_BUILTIN(div, object_int_obj_div, struct int_obj);

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
    RUNTIME_ERROR("Unexpected object tag: %d", val->tag);
  }

  return res;
}

void to_string_k(struct obj *v, struct obj *k, struct env_obj *env) {
  char *res = obj_to_string_internal(v);

  OBJECT_STRING_OBJ_NEW(result_str, res);

  free(res);

  call_closure_one(k, (struct obj *)&result_str);

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
