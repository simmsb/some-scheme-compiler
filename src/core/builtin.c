#include "base.h"
#include "builtin.h"
#include "common.h"

struct int_obj object_int_obj_add(struct object *lhs, struct object *rhs) {
  if (DEBUG_ONLY(lhs->tag != OBJ_INT))
    RUNTIME_ERROR("Left operand to binary add not of integer type");
  if (DEBUG_ONLY(rhs->tag != OBJ_INT))
    RUNTIME_ERROR("Right operand to binary add not of integer type");

  struct int_obj *lhs_int = (struct int_obj *)lhs;
  struct int_obj *rhs_int = (struct int_obj *)rhs;

  return object_int_obj_new(lhs_int->val + rhs_int->val);
}

struct int_obj object_int_obj_sub(struct object *lhs, struct object *rhs) {
  if (DEBUG_ONLY(lhs->tag != OBJ_INT))
    RUNTIME_ERROR("Left operand to binary sub not of integer type");
  if (DEBUG_ONLY(rhs->tag != OBJ_INT))
    RUNTIME_ERROR("Right operand to binary sub not of integer type");

  struct int_obj *lhs_int = (struct int_obj *)lhs;
  struct int_obj *rhs_int = (struct int_obj *)rhs;

  return object_int_obj_new(lhs_int->val - rhs_int->val);
}

struct int_obj object_int_obj_mul(struct object *lhs, struct object *rhs) {
  if (DEBUG_ONLY(lhs->tag != OBJ_INT))
    RUNTIME_ERROR("Left operand to binary mul not of integer type");
  if (DEBUG_ONLY(rhs->tag != OBJ_INT))
    RUNTIME_ERROR("Right operand to binary mul not of integer type");

  struct int_obj *lhs_int = (struct int_obj *)lhs;
  struct int_obj *rhs_int = (struct int_obj *)rhs;

  return object_int_obj_new(lhs_int->val * rhs_int->val);
}

struct int_obj object_int_obj_div(struct object *lhs, struct object *rhs) {
  if (DEBUG_ONLY(lhs->tag != OBJ_INT))
    RUNTIME_ERROR("Left operand to binary div not of integer type");
  if (DEBUG_ONLY(rhs->tag != OBJ_INT))
    RUNTIME_ERROR("Right operand to binary div not of integer type");

  struct int_obj *lhs_int = (struct int_obj *)lhs;
  struct int_obj *rhs_int = (struct int_obj *)rhs;

  return object_int_obj_new(lhs_int->val / rhs_int->val);
}

MAKE_TWO_ARG_FROM_BUILTIN(object_int_obj_add, struct int_obj,
                          object_int_obj_add_param, object_int_obj_add_param_2,
                          object_int_obj_add_env, object_int_obj_add_env_2)
MAKE_TWO_ARG_FROM_BUILTIN(object_int_obj_sub, struct int_obj,
                          object_int_obj_sub_param, object_int_obj_sub_param_2,
                          object_int_obj_sub_env, object_int_obj_sub_env_2)
MAKE_TWO_ARG_FROM_BUILTIN(object_int_obj_mul, struct int_obj,
                          object_int_obj_mul_param, object_int_obj_mul_param_2,
                          object_int_obj_mul_env, object_int_obj_mul_env_2)
MAKE_TWO_ARG_FROM_BUILTIN(object_int_obj_div, struct int_obj,
                          object_int_obj_div_param, object_int_obj_div_param_2,
                          object_int_obj_div_env, object_int_obj_div_env_2)

void halt_func_func(struct object *cont, struct env_elem *env) {
  halt_func(cont);
}

char *obj_to_string_internal(struct object *val) {
  char *res;

  switch (val->tag) {
  case OBJ_CLOSURE:
    ALLOC_SPRINTF(res, "closure|%ld", ((struct closure *)val)->env_id);
    break;
  case OBJ_ENV:
    ALLOC_SPRINTF(res, "env|%ld", ((struct env_elem *)val)->ident_id);
    break;
  case OBJ_INT:
    ALLOC_SPRINTF(res, "%d", ((struct int_obj *)val)->val);
    break;
  case OBJ_STR:
    ALLOC_SPRINTF(res, "%s", ((struct string_obj *)val)->buf);
    break;
  case OBJ_VOID:
    ALLOC_SPRINTF(res, "()");
    break;
  default:
    RUNTIME_ERROR("Unexpected object tag: %d", val->tag);
  }

  return res;
}

void to_string_func_func(struct object *val, struct object *cont,
                         struct env_elem *env) {
  char *res = obj_to_string_internal(val);

  OBJECT_STRING_OBJ_NEW(res, result_str);

  free(res);

  call_closure_one(cont, (struct object *)&result_str);
}

void println_func_func(struct object *val, struct object *cont,
                       struct env_elem *env) {
  char *res = obj_to_string_internal(val);

  printf("%s\n", res);

  free(res);

  OBJECT_VOID_OBJ_NEW(temp_void);

  call_closure_one(cont, temp_void);
}
