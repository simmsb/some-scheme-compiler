#ifndef SOMESCHEME_H
#define SOMESCHEME_H

#include <stdlib.h>
#include <stdbool.h>

#define RUNTIME_ERROR(S) do { fprintf(stderr, "Runtime Error (%s:%d): %s\n", __func__, __LINE__, (S)); exit(1); } while (0)

enum closure_size {
    CLOSURE_ONE = 0,
    CLOSURE_TWO,
};

enum object_tag {
    CLOSURE = 0,
    ENV,
};

enum gc_mark_type {
    WHITE = 0,
    GREY,
    BLACK
};

struct object {
    enum object_tag tag;
    enum gc_mark_type mark;
    bool on_stack;
};

struct env_elem {
    struct object base;
    const size_t ident_id;
    struct object * val;    // shared
    struct env_elem * next; // owned by env_elem
};

struct closure {
    struct object base;
    const enum closure_size size;
    const size_t env_id;
    union {
        void (*const fn_1)(struct object *, struct env_elem *);
        void (*const fn_2)(struct object *, struct object *, struct env_elem *);
    };
    struct env_elem *env;
};

struct env_table_entry {
    const size_t env_id;
    const size_t num_ids;
    size_t * const var_ids;
};


// The map of env ids to an array of var ids
extern struct env_table_entry env_table[];

struct thunk {
    struct closure *closr;
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
void run_minor_gc(struct thunk *);

struct object object_base_new(enum object_tag);

#endif /* SOMESCHEME_H */
