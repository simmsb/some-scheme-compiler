#include <stdlib.h>
#include <stdbool.h>

#ifndef SOMESCHEME_H
#define SOMESCHEME_H

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
            struct object *rator;
            size_t rand_id;
            struct object *rand;
        } one;
        struct {
            struct object *rator;
            size_t rand_id;
            struct object *rand;
            size_t cont_id;
            struct object *cont;
        } two;
    };
};

void call_closure_one(struct object *, size_t, struct object *);
void call_closure_two(struct object *, size_t, struct object *, size_t, struct object *);
void halt_func(struct object *, struct env_elem *);
void scheme_init(void);
void run_gc(struct thunk *);

#endif /* SOMESCHEME_H */
