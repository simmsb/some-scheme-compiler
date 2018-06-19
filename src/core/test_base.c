#include "base.h"

struct env_table_entry global_env_table[] = {
    ENV_ENTRY(0, 0),
    ENV_ENTRY(0, 0, 1),
};


int main() {
    struct env_elem env = {
        .base = object_base_new(ENV),
        .ident_id = 0,
        .val = NULL,
        .prev = NULL,
        .nexts = vector_env_elem_nexts_new(0),
    };


    struct closure closr = {
        .base = object_base_new(CLOSURE),
        .size = CLOSURE_ONE,
        .env_id = 0,
        .fn_1 = halt_func,
        .env = &env,
    };

    struct thunk thnk = {
        .closr = &closr,
        .one.rand = NULL,
    };

    struct thunk *heap_thnk = malloc(sizeof(struct thunk));
    memcpy(heap_thnk, &thnk, sizeof(struct thunk));

    scheme_start(heap_thnk);
}
