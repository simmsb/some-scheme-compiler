#ifndef SOMESCHEME_H
#define SOMESCHEME_H

#include <stdlib.h>
#include <stdbool.h>

#define RUNTIME_ERROR(S) do { fprintf(stderr, "Runtime Error (%s:%d): %s\n", __func__, __LINE__, (S)); exit(1); } while (0)

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
    size_t env_id;
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

struct thunk {
    bool size;
    struct closure closr;
    union {
        struct {
            struct object *rand;
        } one;
        struct {
            struct object *rand;
            struct object *cont;
        } two;
    };
};

void call_closure_one(struct object *, size_t, struct object *);
void call_closure_two(struct object *, size_t, struct object *, size_t, struct object *);
void halt_func(struct object *, struct env_elem *);
void scheme_start(struct thunk *);
void run_gc(struct thunk *);

#endif /* SOMESCHEME_H */
