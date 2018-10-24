#include "base.h"
#include "builtin.h"

void lambda_3(struct object *, struct env_elem *);
void lambda_0(struct object *, struct env_elem *);
void lambda_5(struct object *, struct env_elem *);
void lambda_4(struct object *, struct env_elem *);
void lambda_2(struct object *, struct env_elem *);
void lambda_1(struct object *, struct env_elem *);
void lambda_3(struct object *$anon_var_operator_var_0, struct env_elem *env) {
  ADD_ENV(3, $anon_var_operator_var_0, &(env));
  struct closure unique_var_0 = (object_closure_one_new)(4, lambda_4, env);
  OBJECT_INT_OBJ_NEW(2, unique_var_1);
  (call_closure_one)((struct object *)(&(unique_var_0)), unique_var_1);
}
void lambda_0(struct object *$anon_var_operator_var_3, struct env_elem *env) {
  ADD_ENV(0, $anon_var_operator_var_3, &(env));
  struct closure unique_var_0 = (object_closure_one_new)(1, lambda_1, env);
  OBJECT_INT_OBJ_NEW(1, unique_var_1);
  (call_closure_one)((struct object *)(&(unique_var_0)), unique_var_1);
}
void lambda_5(struct object *$anon_var_rv_2, struct env_elem *env) {
  ADD_ENV(5, $anon_var_rv_2, &(env));
  struct closure unique_var_0 =
      (object_closure_two_new)(halt_func_env, halt_func_func, env);
  (call_closure_one)((struct object *)(&(unique_var_0)), (env_get)(5, env));
}
void lambda_4(struct object *$anon_var_operand_var_1, struct env_elem *env) {
  ADD_ENV(4, $anon_var_operand_var_1, &(env));
  struct closure unique_var_0 = (object_closure_one_new)(5, lambda_5, env);
  (call_closure_two)((env_get)(3, env), (env_get)(4, env),
                     (struct object *)(&(unique_var_0)));
}
void lambda_2(struct object *$anon_var_rv_5, struct env_elem *env) {
  ADD_ENV(2, $anon_var_rv_5, &(env));
  struct closure unique_var_0 = (object_closure_one_new)(3, lambda_3, env);
  (call_closure_one)((struct object *)(&(unique_var_0)), (env_get)(2, env));
}
void lambda_1(struct object *$anon_var_operand_var_4, struct env_elem *env) {
  ADD_ENV(1, $anon_var_operand_var_4, &(env));
  struct closure unique_var_0 = (object_closure_one_new)(2, lambda_2, env);
  (call_closure_two)((env_get)(0, env), (env_get)(1, env),
                     (struct object *)(&(unique_var_0)));
}
void main_lambda(struct object *_, struct env_elem *env) {
  struct closure unique_var_0 = (object_closure_one_new)(0, lambda_0, env);
  struct closure unique_var_1 = (object_closure_two_new)(
      object_int_obj_add_env, object_int_obj_add_func, env);
  (call_closure_one)((struct object *)(&(unique_var_0)),
                     (struct object *)(&(unique_var_1)));
}
struct env_table_entry(global_env_table)[] = {ENV_ENTRY(6, 6),
                                              ENV_ENTRY(7, 6, 7),
                                              ENV_ENTRY(8, 8),
                                              ENV_ENTRY(9, 9, 8),
                                              ENV_ENTRY(10, 10),
                                              ENV_ENTRY(11, 10, 11),
                                              ENV_ENTRY(12, 12),
                                              ENV_ENTRY(13, 13, 12),
                                              ENV_ENTRY(14, 14),
                                              ENV_ENTRY(0, 0),
                                              ENV_ENTRY(1, 0, 1),
                                              ENV_ENTRY(2, 0, 1, 2),
                                              ENV_ENTRY(3, 0, 1, 2, 3),
                                              ENV_ENTRY(4, 0, 1, 2, 3, 4),
                                              ENV_ENTRY(5, 0, 1, 2, 3, 5, 4)};
size_t object_int_obj_add_param = 6;
size_t object_int_obj_add_param_2 = 7;
size_t object_int_obj_sub_param = 8;
size_t object_int_obj_sub_param_2 = 9;
size_t object_int_obj_mul_param = 10;
size_t object_int_obj_mul_param_2 = 11;
size_t object_int_obj_div_param = 12;
size_t object_int_obj_div_param_2 = 13;
size_t halt_func_param = 14;
size_t object_int_obj_add_env = 6;
size_t object_int_obj_add_env_2 = 7;
size_t object_int_obj_sub_env = 8;
size_t object_int_obj_sub_env_2 = 9;
size_t object_int_obj_mul_env = 10;
size_t object_int_obj_mul_env_2 = 11;
size_t object_int_obj_div_env = 12;
size_t object_int_obj_div_env_2 = 13;
size_t halt_func_env = 14;

int main() {
  // TODO: do we need an initial empty env or something
  // thonk it

  struct env_elem base_env = {
    .base = object_base_new(OBJ_ENV),
    .ident_id = 0,
    .val = NULL,
    .nexts = vector_env_elem_nexts_new(0),
  };

  struct closure initial_closure = object_closure_one_new(0, main_lambda, &base_env);
  struct thunk initial_thunk = {
    .closr = &initial_closure,
    .one = {NULL},
  };

  struct thunk *thnk_heap = malloc(sizeof(struct thunk));
  memcpy(thnk_heap, &initial_thunk, sizeof(struct thunk));
  scheme_start(thnk_heap);
}
