#ifndef SOMESCHEME_H
#define SOMESCHEME_H

#include <stdbool.h>
#include <stdlib.h>

#include "common.h"
#include "vec.h"

#define ADD_ENV(IDENT, VAL, HEAD_PTR)                                          \
    do {                                                                       \
        struct env_elem *new_env = alloca(sizeof(struct env_elem));            \
        memcpy(new_env,                                                        \
               &(struct env_elem){.base = object_base_new(OBJ_ENV),            \
                                  .ident_id = (IDENT),                         \
                                  .val = (VAL),                                \
                                  .prev = *(HEAD_PTR),                         \
                                  .nexts = vector_env_elem_nexts_new(0)},      \
               sizeof(struct env_elem));                                       \
                                                                               \
        vector_env_elem_nexts_push(&(*HEAD_PTR)->nexts, new_env);              \
        (*HEAD_PTR) = new_env;                                                 \
    } while (0)

#define NUM_ARGS(...) (sizeof((size_t[]){__VA_ARGS__}) / sizeof(size_t))
#define ENV_ENTRY(ID, ...)                                                     \
    [ID] = (struct env_table_entry) {                                          \
        ID, NUM_ARGS(__VA_ARGS__), (size_t[]) { __VA_ARGS__ }                  \
    }

#define OBJECT_STRING_OBJ_NEW(S, STORE_LOC)                                    \
    do {                                                                       \
        size_t len = strlen(S);                                                \
        struct string_obj *new_obj = alloca(sizeof(struct string_obj) + len);  \
        new_obj->base = object_base_new(OBJ_STR);                              \
        new_obj->len = len;                                                    \
        memcpy(&new_obj->buf, (S), len);                                       \
        *(STORE_LOC) = new_obj;                                                \
    } while (0)

DEFINE_VECTOR(struct env_elem *, env_elem_nexts)

enum closure_size {
    CLOSURE_ONE = 0,
    CLOSURE_TWO,
};

enum object_tag {
    OBJ_CLOSURE = 0,
    OBJ_ENV,
    OBJ_INT,
    OBJ_VOID,
    OBJ_STR,
};

enum gc_mark_type { WHITE = 0, GREY, BLACK };

struct object {
    enum object_tag tag;
    enum gc_mark_type mark;
    bool on_stack;
};

// builtin objects

struct env_elem {
    struct object base;
    const size_t ident_id;
    struct object *val; // shared
    struct env_elem *prev;
    struct vector_env_elem_nexts nexts;
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

struct int_obj {
    struct object base;
    int64_t val;
};

struct void_obj {
    struct object base;
};

struct string_obj {
    struct object base;
    size_t len;
    const char buf[];
};

struct env_table_entry {
    const size_t env_id;
    const size_t num_ids;
    size_t *const var_ids;
};

// get an object from the environment
struct object *env_get(size_t, struct env_elem *);

// set an existing value in the environment, returning the previous value
struct object *env_set(size_t, struct env_elem *, struct object *);

// The map of env ids to an array of var ids
extern struct env_table_entry global_env_table[];

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

void call_closure_one(struct object *, struct object *);
void call_closure_two(struct object *, struct object *, struct object *);
struct void_obj halt_func(struct object *);
void scheme_start(struct thunk *);
void run_minor_gc(struct thunk *);

struct object object_base_new(enum object_tag);
struct closure object_closure_one_new(size_t,
                                      void (*const)(struct object *,
                                                    struct env_elem *),
                                      struct env_elem *);
struct closure object_closure_two_new(
    size_t, void (*const)(struct object *, struct object *, struct env_elem *),
    struct env_elem *);
struct int_obj object_int_obj_new(int64_t);
struct void_obj object_void_obj_new(void);
struct string_obj object_string_obj_new(const char *);

#endif /* SOMESCHEME_H */
