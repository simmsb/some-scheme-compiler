#include "builtin.h"
#include "base.h"

struct int_obj object_int_obj_add(struct object *lhs, struct object *rhs) {
    if (lhs->tag != OBJ_INT)
        RUNTIME_ERROR("Left operand to binary add not of integer type");
    if (rhs->tag != OBJ_INT)
        RUNTIME_ERROR("Right operand to binary add not of integer type");

    struct int_obj *lhs_int = (struct int_obj *)lhs;
    struct int_obj *rhs_int = (struct int_obj *)rhs;

    return object_int_obj_new(lhs_int->val + rhs_int->val);
}

struct int_obj object_int_obj_sub(struct object *lhs, struct object *rhs) {
    if (lhs->tag != OBJ_INT)
        RUNTIME_ERROR("Left operand to binary sub not of integer type");
    if (rhs->tag != OBJ_INT)
        RUNTIME_ERROR("Right operand to binary sub not of integer type");

    struct int_obj *lhs_int = (struct int_obj *)lhs;
    struct int_obj *rhs_int = (struct int_obj *)rhs;

    return object_int_obj_new(lhs_int->val - rhs_int->val);
}

struct int_obj object_int_obj_mul(struct object *lhs, struct object *rhs) {
    if (lhs->tag != OBJ_INT)
        RUNTIME_ERROR("Left operand to binary mul not of integer type");
    if (rhs->tag != OBJ_INT)
        RUNTIME_ERROR("Right operand to binary mul not of integer type");

    struct int_obj *lhs_int = (struct int_obj *)lhs;
    struct int_obj *rhs_int = (struct int_obj *)rhs;

    return object_int_obj_new(lhs_int->val * rhs_int->val);
}

struct int_obj object_int_obj_div(struct object *lhs, struct object *rhs) {
    if (lhs->tag != OBJ_INT)
        RUNTIME_ERROR("Left operand to binary div not of integer type");
    if (rhs->tag != OBJ_INT)
        RUNTIME_ERROR("Right operand to binary div not of integer type");

    struct int_obj *lhs_int = (struct int_obj *)lhs;
    struct int_obj *rhs_int = (struct int_obj *)rhs;

    return object_int_obj_new(lhs_int->val / rhs_int->val);
}

MAKE_TWO_ARG_FROM_BUILTIN(object_int_obj_add, struct int_obj,
                          object_int_obj_add_param,
                          object_int_obj_add_param_2, object_int_obj_add_env,
                          object_int_obj_add_env_2)
MAKE_TWO_ARG_FROM_BUILTIN(object_int_obj_sub, struct int_obj,
                          object_int_obj_sub_param,
                          object_int_obj_sub_param_2, object_int_obj_sub_env,
                          object_int_obj_sub_env_2)
MAKE_TWO_ARG_FROM_BUILTIN(object_int_obj_mul, struct int_obj,
                          object_int_obj_mul_param,
                          object_int_obj_mul_param_2, object_int_obj_mul_env,
                          object_int_obj_mul_env_2)
MAKE_TWO_ARG_FROM_BUILTIN(object_int_obj_div, struct int_obj,
                          object_int_obj_div_param,
                          object_int_obj_div_param_2, object_int_obj_div_env,
                          object_int_obj_div_env_2)

MAKE_ONE_ARG_FROM_BUILTIN(halt_func, struct void_obj, halt_func_param, halt_func_env)
