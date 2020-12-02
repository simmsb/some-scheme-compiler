#include <setjmp.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>

#include "base.h"
#include "common.h"
#include "gc.h"
#include "hash_table.h"
#include "vec.h"

static bool stack_check(void);

static struct thunk *current_thunk;
static void *stack_initial;
static jmp_buf setjmp_env_buf;

void call_closure_one(struct obj *rator, struct obj *rand) {
  if (rator->tag != OBJ_CLOSURE) {
    RUNTIME_ERROR("Called object (%p) was not a closure but was: %d", rator,
                  rator->tag);
  }

  struct closure_obj *closure = (struct closure_obj *)rator;

  if (closure->size != CLOSURE_ONE) {
    printf("Trying to call: %p\n", closure->fn_1);
    RUNTIME_ERROR("Called a closure that takes two args with one arg");
  }

  if (stack_check()) {
    closure->fn_1(rand, closure->env);
  } else {
    // TODO: move to our own gc allocator?
    struct thunk *thnk_heap = malloc(sizeof(struct thunk));
    thnk_heap->closr = closure;
    thnk_heap->one.rand = rand;
    run_minor_gc(thnk_heap);
  }
}

void call_closure_two(struct obj *rator, struct obj *rand, struct obj *cont) {
  if (rator->tag != OBJ_CLOSURE) {
    RUNTIME_ERROR("Called object (%p) was not a closure but was: %d", rator,
                  rator->tag);
  }

  struct closure_obj *closure = (struct closure_obj *)rator;

  if (closure->size != CLOSURE_TWO) {
    RUNTIME_ERROR("Called a closure that takes one arg with two args");
  }

  if (stack_check()) {
    closure->fn_2(rand, cont, closure->env);
  } else {
    // TODO: move to our own gc allocator?
    struct thunk *thnk_heap = malloc(sizeof(struct thunk));
    thnk_heap->closr = closure;
    thnk_heap->two.rand = rand;
    thnk_heap->two.cont = cont;
    run_minor_gc(thnk_heap);
  }
}

static size_t get_stack_limit(void) {
  static size_t cached_limit = 0;

  if (cached_limit != 0) {
    return cached_limit;
  }

  struct rlimit limit;
  getrlimit(RLIMIT_STACK, &limit);
  cached_limit = limit.rlim_cur;
  return cached_limit;
}

static void *stack_ptr(void) { return __builtin_frame_address(0); }

/*
 * Are we above the stack limit
 */
static bool stack_check(void) {
  // buffer area at the end of the stack since idk how accurate this is
  // so reserve 256K for anything we might do after getting to the 'limit'
  static size_t stack_buffer = 1024 * 256;
  uintptr_t stack_ptr_val = (uintptr_t)stack_ptr();
  uintptr_t stack_end_val =
      (uintptr_t)(stack_initial - get_stack_limit() + stack_buffer);

  return stack_ptr_val > stack_end_val;
}

void scheme_start(struct thunk *initial_thunk) {
  stack_initial = stack_ptr();
  current_thunk = initial_thunk;

  gc_init();

  // This is our trampoline, when we come back from a longjmp a different
  // current_thunk will be set and we will just trampoline into the new
  // thunk
  setjmp(setjmp_env_buf);

  DEBUG_FPRINTF(stderr, "bouncing\n");

  if (current_thunk->closr->size == CLOSURE_ONE) {
    struct closure_obj *closr = current_thunk->closr;
    struct obj *rand = current_thunk->one.rand;
    struct env_obj *env = current_thunk->closr->env;
    free(current_thunk);
    closr->fn_1(rand, env);
  } else {
    struct closure_obj *closr = current_thunk->closr;
    struct obj *rand = current_thunk->two.rand;
    struct obj *cont = current_thunk->two.cont;
    struct env_obj *env = current_thunk->closr->env;
    free(current_thunk);
    closr->fn_2(rand, cont, env);
  }

  RUNTIME_ERROR("Control flow returned from trampoline function.");
}

void run_minor_gc(struct thunk *thnk) {
  current_thunk = thnk;

  struct gc_context ctx = gc_make_context();
  gc_minor(&ctx, thnk);
  gc_free_context(&ctx);

  // Jump back to the start
  longjmp(setjmp_env_buf, 1);
}

struct obj object_base_new(enum object_tag tag) {
  return (struct obj){
      .tag = tag,
      .mark = WHITE,
      .on_stack = true,
#ifdef DEBUG
      .last_touched_by = "object_init",
#endif
  };
}

struct closure_obj object_closure_one_new(void (*fn)(struct obj *,
                                                     struct env_obj *),
                                          struct env_obj *env) {
  return (struct closure_obj){.base = object_base_new(OBJ_CLOSURE),
                              .size = CLOSURE_ONE,
                              .fn_1 = fn,
                              .env = env};
}

struct closure_obj object_closure_two_new(void (*fn)(struct obj *, struct obj *,
                                                     struct env_obj *),
                                          struct env_obj *env) {
  return (struct closure_obj){.base = object_base_new(OBJ_CLOSURE),
                              .size = CLOSURE_TWO,
                              .fn_2 = fn,
                              .env = env};
}

struct int_obj object_int_obj_new(int64_t val) {
  return (struct int_obj){.base = object_base_new(OBJ_INT), .val = val};
}

struct cons_obj object_cons_obj_new(struct obj *car, struct obj *cdr) {
  return (struct cons_obj){
      .base = object_base_new(OBJ_CONS), .car = car, .cdr = cdr};
}

static size_t hash_string(const char *buf, size_t len) {
  size_t hash = 14695981039346656037ull;

  for (size_t i = 0; i < len; i++) {
    hash ^= (size_t)buf[i];
    hash *= 1099511628211;
  }

  return hash;
}

static size_t hash_combine(size_t a, size_t b) {
  size_t hash = a;

  hash *= 1099511628211;
  hash ^= b;
  hash *= 1099511628211;

  return hash;
}

size_t hash_obj_impl(struct obj *obj) {
  if (!obj) {
    return 0;
  }

  switch (obj->tag) {
  case OBJ_INT:
    return ((struct int_obj *)obj)->val;
  case OBJ_STR: {
    struct string_obj *str_obj = (struct string_obj *)obj;
    return hash_string(str_obj->buf, str_obj->len);
  }
  case OBJ_CONS: {
    struct cons_obj *cons_obj = (struct cons_obj *)obj;
    size_t a = hash_obj_impl(cons_obj->car);
    size_t b = hash_obj_impl(cons_obj->cdr);
    return hash_combine(a, b);
  }
  case OBJ_HT: {
    struct ht_obj *ht_obj = (struct ht_obj *)obj;
    size_t hash = 14695981039346656037ull;

    HASH_TABLE_ITER(obj, key, val, ht_obj->ht, {
      hash = hash_combine(hash, hash_obj_impl(*key));
      hash = hash_combine(hash, hash_obj_impl(*val));
    });

    return hash;
  }
  case OBJ_CELL:
    return hash_obj_impl(((struct cell_obj *)obj)->val);
  default:
    RUNTIME_ERROR("Unhashable type: %d", obj->tag);
  }
}

bool eq_obj_impl(struct obj *a, struct obj *b) {
  if (!a && !b)
    return true;

  if (!a || !b)
    return false;

  if (a->tag != b->tag)
    return false;

  switch (a->tag) {
  case OBJ_INT:
    return ((struct int_obj *)a)->val == ((struct int_obj *)b)->val;
  case OBJ_STR: {
    struct string_obj *str_obj_a = (struct string_obj *)a;
    struct string_obj *str_obj_b = (struct string_obj *)b;

    if (str_obj_a->len != str_obj_b->len)
      return false;

    return strncmp(str_obj_a->buf, str_obj_b->buf, str_obj_a->len) == 0;
  }
  case OBJ_CONS: {
    struct cons_obj *cons_obj_a = (struct cons_obj *)a;
    struct cons_obj *cons_obj_b = (struct cons_obj *)b;
    return eq_obj_impl(cons_obj_a->car, cons_obj_b->car) &&
           eq_obj_impl(cons_obj_a->cdr, cons_obj_b->cdr);
  }
  case OBJ_HT: {
    struct ht_obj *ht_obj_a = (struct ht_obj *)a;
    struct ht_obj *ht_obj_b = (struct ht_obj *)b;

    HASH_TABLE_ITER(obj, key, val, ht_obj_a->ht, {
      typeof(*val) *b_val = hash_table_obj_lookup(ht_obj_b->ht, *key);

      if (!b_val)
        return false;

      if (!eq_obj_impl(*val, *b_val))
        return false;
    });

    HASH_TABLE_ITER(obj, key, val, ht_obj_b->ht, {
      typeof(*val) *a_val = hash_table_obj_lookup(ht_obj_a->ht, *key);

      if (!a_val)
        return false;

      // we don't need to check equality of values this time
    });

    return true;
  }
  case OBJ_CELL:
    return eq_obj_impl(((struct cell_obj *)a)->val,
                       ((struct cell_obj *)b)->val);
  default:
    return false;
  }
}

MAKE_HASH(struct obj *, struct obj *, hash_obj_impl, eq_obj_impl, obj);

struct ht_obj object_ht_obj_new() {
  struct hash_table_obj *ht = hash_table_obj_new();

  return (struct ht_obj){.base = object_base_new(OBJ_HT), .ht = ht};
}
