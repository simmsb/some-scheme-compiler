#ifndef SOMESCHEME_H
#define SOMESCHEME_H

#include <stdbool.h>
#include <stdlib.h>

#include "common.h"
#include "vec.h"

#define NUM_ARGS(...) (sizeof((size_t[]){__VA_ARGS__}) / sizeof(size_t))

#define OBJECT_STRING_OBJ_NEW(NAME, S)                                         \
  struct obj *(NAME);                                                          \
  do {                                                                         \
    size_t len = strlen(S) + 1;                                                \
    /* we keep the null byte */                                                \
    struct string_obj *new_obj = alloca(sizeof(struct string_obj) + len);      \
    new_obj->base = object_base_new(OBJ_STR);                                  \
    new_obj->len = len;                                                        \
    memcpy((char *)&new_obj->buf, (S), len);                                   \
    TOUCH_OBJECT(new_obj, "string_obj_new");                                   \
    (NAME) = (struct obj *)new_obj;                                            \
  } while (0)

#define OBJECT_INT_OBJ_NEW(NAME, n)                                            \
  struct obj *(NAME);                                                          \
  do {                                                                         \
    struct int_obj *new_obj = alloca(sizeof(struct int_obj));                  \
    *new_obj = object_int_obj_new((n));                                        \
    TOUCH_OBJECT(new_obj, "int_obj_new");                                      \
    (NAME) = (struct obj *)new_obj;                                            \
  } while (0)

#define ENV_STRUCT(T)                                                          \
  struct {                                                                     \
    struct obj base;                                                           \
    size_t len;                                                                \
    T env;                                                                     \
  }

#define OBJECT_ENV_OBJ_NEW(NAME, S)                                            \
  struct env_obj *(NAME);                                                      \
  do {                                                                         \
    ENV_STRUCT(S) *new_env = alloca(sizeof(ENV_STRUCT(S)));                    \
    new_env->base = object_base_new(ENV_OBJ);                                  \
    new_env->len = sizeof(S) / sizeof(struct obj *);                           \
    memset(&new_env->env, 0, sizeof(S));                                       \
    (NAME) = (struct env_obj *)new_env;                                        \
  } while (0)

#define OBJECT_CLOSURE_ONE_NEW(NAME, FN, ENV)                                  \
  struct obj *(NAME);                                                          \
  do {                                                                         \
    struct closure_obj *new_obj = alloca(sizeof(struct closure_obj));          \
    *new_obj = object_closure_one_new((FN), (ENV));                            \
    TOUCH_OBJECT(new_obj, "closure_one_new");                                  \
    (NAME) = (struct obj *)new_obj;                                            \
  } while (0)

#define OBJECT_CLOSURE_TWO_NEW(NAME, FN, ENV)                                  \
  struct obj *(NAME);                                                          \
  do {                                                                         \
    struct closure_obj *new_obj = alloca(sizeof(struct closure_obj));          \
    *new_obj = object_closure_two_new((FN), (ENV));                            \
    TOUCH_OBJECT(new_obj, "closure_two_new");                                  \
    (NAME) = (struct obj *)new_obj;                                            \
  } while (0)

#define OBJECT_CELL_OBJ_NEW(NAME, VAL)                                         \
  struct obj *(NAME);                                                          \
  do {                                                                         \
    struct cell_obj *new_obj = alloca(sizeof(struct cell_obj));                \
    new_obj->base = object_base_new(OBJ_CELL);                                 \
    new_obj->val = (VAL);                                                      \
    TOUCH_OBJECT(new_obj, "object_cell_new");                                  \
    (NAME) = (struct obj *)new_obj;                                            \
  } while (0)

#ifdef DEBUG
#define TOUCH_OBJECT(OBJ, S)                                                   \
  do {                                                                         \
    fprintf(stderr,                                                            \
            "touching object %p tag: %d, last touched by %s: (%s:%d:%s)\n",    \
            (void *)(OBJ), ((struct obj *)(OBJ))->tag,                         \
            ((struct obj *)(OBJ))->last_touched_by, __func__, __LINE__, (S));  \
    ALLOC_SPRINTF(((struct obj *)(OBJ))->last_touched_by, "(%s:%d:%s)",        \
                  __func__, __LINE__, (S));                                    \
  } while (0)
#else
#define TOUCH_OBJECT(OBJ, S)                                                   \
  do {                                                                         \
  } while (0)
#endif // DEBUG

enum __attribute__((__packed__)) closure_size {
  CLOSURE_ONE = 0,
  CLOSURE_TWO,
};

enum __attribute__((__packed__)) object_tag {
  OBJ_CLOSURE = 1,
  ENV_OBJ,
  OBJ_INT,
  OBJ_STR,
  OBJ_CELL,
};

#define LAST_OBJ_TYPE OBJ_CELL

enum __attribute__((__packed__)) gc_mark_type { WHITE = 0, GREY, BLACK };

struct obj {
  enum object_tag tag;
  enum gc_mark_type mark;
  bool on_stack;
#ifdef DEBUG
  char *last_touched_by;
#endif
};

// builtin objects

struct env_obj {
  struct obj base;
  size_t len;
  struct obj *env[];
};

struct cell_obj {
  struct obj base;
  struct obj *val;
};

struct closure_obj {
  struct obj base;
  enum closure_size size;
  union {
    void (*fn_1)(struct obj *, struct env_obj *);
    void (*fn_2)(struct obj *, struct obj *, struct env_obj *);
  };
  struct env_obj *env;
};

struct int_obj {
  struct obj base;
  int64_t val;
};

struct string_obj {
  struct obj base;
  size_t len;
  const char buf[];
};

struct thunk {
  struct closure_obj *closr;
  union {
    struct {
      struct obj *rand;
    } one;
    struct {
      struct obj *rand;
      struct obj *cont;
    } two;
  };
};

void call_closure_one(struct obj *, struct obj *);
void call_closure_two(struct obj *, struct obj *, struct obj *);
void scheme_start(struct thunk *);
void run_minor_gc(struct thunk *);

struct obj object_base_new(enum object_tag);
struct closure_obj object_closure_one_new(void (*)(struct obj *,
                                                   struct env_obj *),
                                          struct env_obj *);
struct closure_obj object_closure_two_new(void (*)(struct obj *, struct obj *,
                                                   struct env_obj *),
                                          struct env_obj *);
struct int_obj object_int_obj_new(int64_t);

#endif /* SOMESCHEME_H */
