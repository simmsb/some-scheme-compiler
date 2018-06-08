#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>

#define RUNTIME_ERROR(S) printf("Runtime Error: %s\n", (S))

enum object_tag {
    CLOSURE = 0,
};

struct object {
    enum object_tag tag;
};

struct env_elem {
    size_t ident_id;
    struct object *val;    // shared
    struct env_elem *next; // owned by env_elem
};

struct closure {
    struct object base;
    bool size; // false = 1 arg, true = 2 arg
    union {
        void (*fn_1)(struct object *, struct env_elem *);
        void (*fn_2)(struct object *, struct object *, struct env_elem *);
    };
    struct env_elem *env;
};

struct env_table_entry {
    size_t env_id;
    size_t *var_ids;
};

extern struct env_table_entry env_table[];

#define ADD_ENV(IDENT, VAL, HEAD)                                       \
    do {                                                                \
        struct env_elem *new_env = alloca(sizeof(struct env_elem));     \
                                                                        \
        new_env->ident_id = (IDENT);                                    \
        new_env->val = (VAL);                                           \
        new_env->next = (HEAD);                                         \
                                                                        \
        (HEAD) = new_env;                                               \
    } while (0)                                                         \

void call_closure_one(struct object *rator, size_t rand_id, struct object *rand) {
    if (rator->tag != 0) {
        RUNTIME_ERROR("Called object was not a closure");
    }

    struct closure *closure = (struct closure *)rator;

    if (closure->size != false) {
        RUNTIME_ERROR("Called a closure that takes two args with one arg");
    }

    ADD_ENV(rand_id, rand, closure->env);
    closure->fn_1(rand, closure->env);
}

void call_closure_two(struct object *rator, size_t rand_id, struct object *rand, size_t cont_id, struct object *cont) {
    if (rator->tag != 0) {
        RUNTIME_ERROR("Called object was not a closure");
    }

    struct closure *closure = (struct closure *)rator;

    if (closure->size != true) {
        RUNTIME_ERROR("Called a closure that takes two args with one arg");
    }

    ADD_ENV(rand_id, rand, closure->env);
    ADD_ENV(cont_id, cont, closure->env);

    closure->fn_2(rand, cont, closure->env);
}

void halt_func(struct object *cont, struct env_elem *env) {
    (void)cont; // mmh
    (void)env;
    printf("Halt");
    exit(0);
}
