#include "base.h"
#include "builtin.h"

void lambda_8(struct object *, struct env_elem *);
void lambda_0(struct object *, struct env_elem *);
void lambda_7(struct object *, struct env_elem *);
void lambda_3(struct object *, struct env_elem *);
void lambda_2(struct object *, struct object *, struct env_elem *);
void lambda_5(struct object *, struct env_elem *);
void lambda_6(struct object *, struct env_elem *);
void lambda_1(struct object *, struct env_elem *);
void lambda_4(struct object *, struct env_elem *);
void lambda_8(struct object *$anon_var_rand_var_5, struct env_elem *env) {
  ADD_ENV(9, $anon_var_rand_var_5, &(env));
  (call_closure_two)((env_get)(4, env), (env_get)(9, env), (env_get)(3, env));
}
void lambda_0(struct object *$anon_var_rator_var_1, struct env_elem *env) {
  ADD_ENV(0, $anon_var_rator_var_1, &(env));
  struct closure unique_var_0 = (object_closure_one_new)(1, lambda_1, env);
  OBJECT_VOID_OBJ_NEW(unique_var_1);
  (call_closure_one)((struct object *)(&(unique_var_0)), unique_var_1);
}
void lambda_7(struct object *$anon_var_rand_var_7, struct env_elem *env) {
  ADD_ENV(8, $anon_var_rand_var_7, &(env));
  struct closure unique_var_0 = (object_closure_one_new)(8, lambda_8, env);
  (call_closure_two)((env_get)(7, env), (env_get)(8, env),
                     (struct object *)(&(unique_var_0)));
}
void lambda_3(struct object *$anon_var_rator_var_4, struct env_elem *env) {
  ADD_ENV(4, $anon_var_rator_var_4, &(env));
  struct closure unique_var_0 = (object_closure_one_new)(4, lambda_4, env);
  struct closure unique_var_1 = (object_closure_two_new)(
      object_int_obj_add_env, object_int_obj_add_func, env);
  (call_closure_one)((struct object *)(&(unique_var_0)),
                     (struct object *)(&(unique_var_1)));
}
void lambda_2(struct object *$throwaway_var_0,
              struct object *$anon_var_cont_var_3, struct env_elem *env) {
  ADD_ENV(2, $throwaway_var_0, &(env));
  ADD_ENV(3, $anon_var_cont_var_3, &(env));
  struct closure unique_var_0 = (object_closure_one_new)(3, lambda_3, env);
  struct closure unique_var_1 =
      (object_closure_two_new)(println_func_env, println_func_func, env);
  (call_closure_one)((struct object *)(&(unique_var_0)),
                     (struct object *)(&(unique_var_1)));
}
void lambda_5(struct object *$anon_var_rand_var_9, struct env_elem *env) {
  ADD_ENV(6, $anon_var_rand_var_9, &(env));
  struct closure unique_var_0 = (object_closure_one_new)(6, lambda_6, env);
  (call_closure_two)((env_get)(5, env), (env_get)(6, env),
                     (struct object *)(&(unique_var_0)));
}
void lambda_6(struct object *$anon_var_rator_var_6, struct env_elem *env) {
  ADD_ENV(7, $anon_var_rator_var_6, &(env));
  struct closure unique_var_0 = (object_closure_one_new)(7, lambda_7, env);
  OBJECT_INT_OBJ_NEW(2, unique_var_1);
  (call_closure_one)((struct object *)(&(unique_var_0)), unique_var_1);
}
void lambda_1(struct object *$anon_var_rand_var_2, struct env_elem *env) {
  ADD_ENV(1, $anon_var_rand_var_2, &(env));
  struct closure unique_var_0 =
      (object_closure_one_new)(halt_func_env, halt_func_func, env);
  (call_closure_two)((env_get)(0, env), (env_get)(1, env),
                     (struct object *)(&(unique_var_0)));
}
void lambda_4(struct object *$anon_var_rator_var_8, struct env_elem *env) {
  ADD_ENV(5, $anon_var_rator_var_8, &(env));
  struct closure unique_var_0 = (object_closure_one_new)(5, lambda_5, env);
  OBJECT_INT_OBJ_NEW(1, unique_var_1);
  (call_closure_one)((struct object *)(&(unique_var_0)), unique_var_1);
}
void main_lambda(struct object *_, struct env_elem *env) {
  struct closure unique_var_0 = (object_closure_one_new)(0, lambda_0, env);
  struct closure unique_var_1 = (object_closure_two_new)(2, lambda_2, env);
  (call_closure_one)((struct object *)(&(unique_var_0)),
                     (struct object *)(&(unique_var_1)));
}
struct env_table_entry(global_env_table)[] = {
    ENV_ENTRY(9, 10),
    ENV_ENTRY(10, 11, 10),
    ENV_ENTRY(11, 12),
    ENV_ENTRY(12, 12, 13),
    ENV_ENTRY(13, 14),
    ENV_ENTRY(14, 14, 15),
    ENV_ENTRY(15, 16),
    ENV_ENTRY(16, 16, 17),
    ENV_ENTRY(17, 18),
    ENV_ENTRY(18, 19),
    ENV_ENTRY(19, 20),
    ENV_ENTRY(0, 0),
    ENV_ENTRY(1, 0, 1),
    ENV_ENTRY(2, 2, 3),
    ENV_ENTRY(3, 4, 2, 3),
    ENV_ENTRY(4, 4, 2, 3, 5),
    ENV_ENTRY(5, 4, 2, 3, 6, 5),
    ENV_ENTRY(6, 7, 4, 2, 3, 6, 5),
    ENV_ENTRY(7, 8, 7, 4, 2, 3, 6, 5),
    ENV_ENTRY(8, 8, 7, 4, 2, 9, 3, 6, 5)};
size_t object_int_obj_add_param = 10;
size_t object_int_obj_add_param_2 = 11;
size_t object_int_obj_sub_param = 12;
size_t object_int_obj_sub_param_2 = 13;
size_t object_int_obj_mul_param = 14;
size_t object_int_obj_mul_param_2 = 15;
size_t object_int_obj_div_param = 16;
size_t object_int_obj_div_param_2 = 17;
size_t halt_func_param = 18;
size_t to_string_func_param = 19;
size_t println_func_param = 20;
size_t object_int_obj_add_env = 9;
size_t object_int_obj_add_env_2 = 10;
size_t object_int_obj_sub_env = 11;
size_t object_int_obj_sub_env_2 = 12;
size_t object_int_obj_mul_env = 13;
size_t object_int_obj_mul_env_2 = 14;
size_t object_int_obj_div_env = 15;
size_t object_int_obj_div_env_2 = 16;
size_t halt_func_env = 17;
size_t to_string_func_env = 18;
size_t println_func_env = 19;

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
