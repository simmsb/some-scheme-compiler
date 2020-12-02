#include <assert.h>
#include <stdbool.h>
#include <string.h>

#include "base.h"
#include "common.h"
#include "gc.h"
#include "hash_table.h"
#include "queue.h"
#include "vec.h"

MAKE_VECTOR(struct obj *, gc_heap_nodes);
MAKE_VECTOR(size_t, size_t);
MAKE_QUEUE(struct obj *, gc_grey_nodes);
MAKE_QUEUE(struct ptr_toupdate_pair, ptr_toupdate_pair);

bool size_t_eq(size_t a, size_t b) { return a == b; }

MAKE_HASH(size_t, struct obj *, hash_table_default_size_t_hash_fun, size_t_eq,
          ptr_map);

static struct gc_data gc_global_data;

// array of gc_funcs for each object type
static struct gc_funcs gc_func_map[] = {
    [OBJ_CLOSURE] = (struct gc_funcs){.toheap = toheap_closure,
                                      .mark = mark_closure,
                                      .free = gc_free_noop},
    [ENV_OBJ] = (struct gc_funcs){.toheap = toheap_env,
                                  .mark = mark_env,
                                  .free = gc_free_noop},
    [OBJ_CELL] = (struct gc_funcs){.toheap = toheap_cell,
                                   .mark = mark_cell,
                                   .free = gc_free_noop},
    [OBJ_CONS] = (struct gc_funcs){.toheap = toheap_cons,
                                   .mark = mark_cons,
                                   .free = gc_free_noop},
    [OBJ_INT] = (struct gc_funcs){.toheap = toheap_int_obj,
                                  .mark = gc_mark_noop,
                                  .free = gc_free_noop},
    [OBJ_STR] =
        (struct gc_funcs){
            .toheap = toheap_string_obj,
            .mark = gc_mark_noop,
            .free = gc_free_noop,
        },
    [OBJ_HT] = (struct gc_funcs){.toheap = toheap_ht,
                                 .mark = mark_ht,
                                 .free = free_ht},
};

// This does nothing, the gc will call free() on the object if it was heap
// allocated
void gc_free_noop(struct obj *obj) { (void)obj; }

// This does nothing, for objects where marking them should mark them as black
// and nothing more
void gc_mark_noop(struct obj *obj, struct gc_context *ctx) {
  (void)obj;
  (void)ctx;
}

// Mark an object as grey and add it to the queue of grey nodes 'if' it is not
// already grey or black
static bool maybe_mark_grey_and_queue(struct gc_context *ctx, struct obj *obj) {
  if (DEBUG_ONLY(!obj)) {
    DEBUG_FPRINTF(stderr, "trying to mark NULL!\n");
  }
  switch (obj->mark) {
  case BLACK:
  case GREY:
    return false;
  case WHITE:
    obj->mark = GREY;
    queue_gc_grey_nodes_enqueue(&ctx->grey_nodes, obj);
    return true;
  }
}

struct obj *toheap_ht(struct obj *ht_obj, struct gc_context *ctx) {
  struct ht_obj *ht = (struct ht_obj *)ht_obj;

  if (ht->base.on_stack) {
    TOUCH_OBJECT(ht, "toheap_ht");
    struct ht_obj *heap_ht = gc_malloc(sizeof(struct ht_obj));
    *heap_ht = *ht;
    ht = heap_ht;
  }

  HASH_TABLE_ITER(obj, key, val, ht->ht, {
    if (*key) {
      struct ptr_toupdate_pair pk = {.toupdate = (struct obj **)key,
                                     .on_stack = (struct obj *)*key};
      queue_ptr_toupdate_pair_enqueue(&ctx->pointers_toupdate, pk);
    }
    if (*val) {
      struct ptr_toupdate_pair pv = {.toupdate = (struct obj **)val,
                                     .on_stack = (struct obj *)*val};
      queue_ptr_toupdate_pair_enqueue(&ctx->pointers_toupdate, pv);
    }
  });

  return (struct obj *)ht;
}

void mark_ht(struct obj *ht_obj, struct gc_context *ctx) {
  struct ht_obj *ht = (struct ht_obj *)ht_obj;

  HASH_TABLE_ITER(obj, key, val, ht->ht, {
    if (*key)
      maybe_mark_grey_and_queue(ctx, *key);
    if (*val)
      maybe_mark_grey_and_queue(ctx, *val);
  });
}

void free_ht(struct obj *ht_obj) {
  struct ht_obj *ht = (struct ht_obj *)ht_obj;

  hash_table_obj_free(ht->ht);
}

struct obj *toheap_cons(struct obj *cons_obj, struct gc_context *ctx) {
  struct cons_obj *cons = (struct cons_obj *)cons_obj;

  if (cons->base.on_stack) {
    TOUCH_OBJECT(cons, "toheap_cons");
    struct cons_obj *heap_cons = gc_malloc(sizeof(struct cons_obj));
    *heap_cons = *cons;
    cons = heap_cons;
  }

  if (cons->car) {
    struct ptr_toupdate_pair p = {.toupdate = (struct obj **)&cons->car,
                                  .on_stack = (struct obj *)cons->car};
    queue_ptr_toupdate_pair_enqueue(&ctx->pointers_toupdate, p);
  }

  if (cons->cdr) {
    struct ptr_toupdate_pair p = {.toupdate = (struct obj **)&cons->cdr,
                                  .on_stack = (struct obj *)cons->cdr};
    queue_ptr_toupdate_pair_enqueue(&ctx->pointers_toupdate, p);
  }

  return (struct obj *)cons;
}

void mark_cons(struct obj *cons_obj, struct gc_context *ctx) {
  struct cons_obj *cons = (struct cons_obj *)cons_obj;

  if (cons->car) {
    maybe_mark_grey_and_queue(ctx, cons->car);
  }

  if (cons->cdr) {
    maybe_mark_grey_and_queue(ctx, cons->cdr);
  }
}

struct obj *toheap_cell(struct obj *cell_obj, struct gc_context *ctx) {
  struct cell_obj *cell = (struct cell_obj *)cell_obj;

  if (cell->base.on_stack) {
    TOUCH_OBJECT(cell, "toheap_cell");
    struct cell_obj *heap_cell = gc_malloc(sizeof(struct cell_obj));
    *heap_cell = *cell;
    cell = heap_cell;
  }

  if (cell->val) {
    struct ptr_toupdate_pair p = {.toupdate = (struct obj **)&cell->val,
                                  .on_stack = (struct obj *)cell->val};
    queue_ptr_toupdate_pair_enqueue(&ctx->pointers_toupdate, p);
  }

  return (struct obj *)cell;
}

void mark_cell(struct obj *cell_obj, struct gc_context *ctx) {
  struct cell_obj *cell = (struct cell_obj *)cell_obj;

  if (cell->val) {
    maybe_mark_grey_and_queue(ctx, cell->val);
  }
}

struct obj *toheap_env(struct obj *env_obj, struct gc_context *ctx) {
  struct env_obj *env = (struct env_obj *)env_obj;
  struct env_obj *orig_env = env;

  if (env->base.on_stack) {
    TOUCH_OBJECT(env, "toheap_env");
    struct env_obj *heap_env =
        gc_malloc(sizeof(struct env_obj) + env->len * sizeof(struct obj *));

    heap_env->base = env->base;
    heap_env->len = env->len;
    memset(&heap_env->env, 0, env->len * sizeof(struct obj *));
    env = heap_env;
  }

  for (size_t i = 0; i < env->len; i++) {
    struct obj *obj_ptr = orig_env->env[i];

    if (!obj_ptr)
      continue;

    struct ptr_toupdate_pair p = {.toupdate = &env->env[i],
                                  .on_stack = obj_ptr};
    queue_ptr_toupdate_pair_enqueue(&ctx->pointers_toupdate, p);
  }

  return (struct obj *)env;
}

void mark_env(struct obj *env_obj, struct gc_context *ctx) {
  struct env_obj *env = (struct env_obj *)env_obj;
  for (size_t i = 0; i < env->len; i++) {
    if (!env->env[i])
      continue;
    maybe_mark_grey_and_queue(ctx, env->env[i]);
  }
}

struct obj *toheap_closure(struct obj *obj, struct gc_context *ctx) {
  struct closure_obj *clos = (struct closure_obj *)obj;

  if (obj->on_stack) {
    TOUCH_OBJECT(obj, "toheap_closure");
    struct closure_obj *heap_clos = gc_malloc(sizeof(struct closure_obj));
    memcpy(heap_clos, obj, sizeof(struct closure_obj));
    clos = heap_clos;
  }

  if (clos->env) {
    struct ptr_toupdate_pair p = {.toupdate = (struct obj **)&clos->env,
                                  .on_stack = (struct obj *)clos->env};
    queue_ptr_toupdate_pair_enqueue(&ctx->pointers_toupdate, p);
  }

  return (struct obj *)clos;
}

void mark_closure(struct obj *obj, struct gc_context *ctx) {
  struct closure_obj *clos = (struct closure_obj *)obj;

  if (clos->env) {
    maybe_mark_grey_and_queue(ctx, (struct obj *)clos->env);
  }
}

struct obj *toheap_int_obj(struct obj *obj, struct gc_context *ctx) {
  struct int_obj *intobj = (struct int_obj *)obj;

  if (obj->on_stack) {
    TOUCH_OBJECT(obj, "toheap_int");
    struct int_obj *heap_intobj = gc_malloc(sizeof(struct int_obj));
    memcpy(heap_intobj, intobj, sizeof(struct int_obj));
    intobj = heap_intobj;
  }

  return (struct obj *)intobj;
}

struct obj *toheap_string_obj(struct obj *obj, struct gc_context *ctx) {
  struct string_obj *strobj = (struct string_obj *)obj;

  if (obj->on_stack) {
    TOUCH_OBJECT(obj, "toheap_string");
    size_t total_size = sizeof(struct string_obj) + strobj->len;

    struct string_obj *heap_stringobj = gc_malloc(total_size);

    memcpy(heap_stringobj, strobj, total_size);

    strobj = heap_stringobj;
  }

  return (struct obj *)strobj;
}

struct gc_context gc_make_context(void) {
  return (struct gc_context){
      .grey_nodes = queue_gc_grey_nodes_new(10),
      .pointers_toupdate = queue_ptr_toupdate_pair_new(10),
      .updated_pointers = hash_table_ptr_map_new(),
  };
}

void gc_free_context(struct gc_context *ctx) {
  queue_gc_grey_nodes_free(&ctx->grey_nodes);
  queue_ptr_toupdate_pair_free(&ctx->pointers_toupdate);
  hash_table_ptr_map_free(ctx->updated_pointers);
}

void gc_mark_obj(struct gc_context *ctx, struct obj *obj) {
  obj->mark = BLACK;
  gc_func_map[obj->tag].mark(obj, ctx);
}

// Moves all live objects on the stack over to the heap
struct obj *gc_toheap(struct gc_context *ctx, struct obj *obj) {
  if (!obj) {
    return NULL;
  }

  // if we've already copied this object,
  // we know that anything it points to must also be sorted
  struct obj **maybe_copied =
      hash_table_ptr_map_lookup(ctx->updated_pointers, (size_t)obj);
  if (maybe_copied != NULL) {
    return *maybe_copied;
  }

  if (DEBUG_ONLY(obj->tag > LAST_OBJ_TYPE)) {
    RUNTIME_ERROR("object %p is corrupted\n", (void *)obj);
  }
  DEBUG_FPRINTF(stderr, "copying object of type: %u to heap\n", obj->tag);

  struct obj *new_obj = gc_func_map[obj->tag].toheap(obj, ctx);

  // mark the object as now being on the heap
  new_obj->on_stack = false;

  // Add it to the updated map
  // Even if it was on the heap already we still insert
  // since we then won't process child objects further
  hash_table_ptr_map_insert(ctx->updated_pointers, (size_t)obj, new_obj);

  return new_obj;
}

// The minor gc, moves all stack objects to the heap
// The parameter 'thnk' is the current thunk holding everything together
// The thunk should be heap allocated and freed after being called
void gc_minor(struct gc_context *ctx, struct thunk *thnk) {
  DEBUG_FPRINTF(stderr, "minor gc occuring\n");

  // initially mark the closure and it's arguments to be applied
  thnk->closr = (struct closure_obj *)gc_toheap(ctx, (struct obj *)thnk->closr);

  switch (thnk->closr->size) {
  case CLOSURE_ONE:
    if (thnk->one.rand != NULL) {
      thnk->one.rand = gc_toheap(ctx, thnk->one.rand);
    }
    break;
  case CLOSURE_TWO:
    if (thnk->two.rand != NULL) {
      thnk->two.rand = gc_toheap(ctx, thnk->two.rand);
    }
    if (thnk->two.cont != NULL) {
      thnk->two.cont = gc_toheap(ctx, thnk->two.cont);
    }
    break;
  }

  // work through each pointer that needs to be updated
  while (queue_ptr_toupdate_pair_len(&ctx->pointers_toupdate) > 0) {
    struct ptr_toupdate_pair to_update =
        queue_ptr_toupdate_pair_dequeue(&ctx->pointers_toupdate);

    struct obj **maybe_copied = hash_table_ptr_map_lookup(
        ctx->updated_pointers, (size_t)to_update.on_stack);

    if (maybe_copied != NULL) {
      // we've already updated this pointer, just update the pointer that
      // needs to be updated
      *to_update.toupdate = *maybe_copied;
    } else {
      // we haven't seen this yet, perform a copy and update

      assert(to_update.on_stack != NULL);

      struct obj *on_heap = gc_toheap(ctx, to_update.on_stack);
      hash_table_ptr_map_insert(ctx->updated_pointers,
                                (size_t)to_update.on_stack, on_heap);
      *to_update.toupdate = on_heap;
    }
  }

  gc_major(ctx, thnk);
}

// The major gc, collects objects on the heap
void gc_major(struct gc_context *ctx, struct thunk *thnk) {
  size_t num_freed = 0;
  size_t num_marked = 0;

  gc_mark_obj(ctx, &thnk->closr->base);
  num_marked++;

  switch (thnk->closr->size) {
  case CLOSURE_ONE:
    if (thnk->one.rand)
      gc_mark_obj(ctx, thnk->one.rand);
    num_marked++;
    break;
  case CLOSURE_TWO:
    if (thnk->two.rand)
      gc_mark_obj(ctx, thnk->two.rand);
    if (thnk->two.cont)
      gc_mark_obj(ctx, thnk->two.cont);
    num_marked++;
    num_marked++;
    break;
  }

  while (queue_gc_grey_nodes_len(&ctx->grey_nodes) > 0) {
    struct obj *next_obj = queue_gc_grey_nodes_dequeue(&ctx->grey_nodes);
    if (DEBUG_ONLY(!next_obj)) {
      RUNTIME_ERROR("NULL was added to mark queue!");
    }
    gc_mark_obj(ctx, next_obj);
    num_marked++;
  }

  DEBUG_FPRINTF("marked %zu objects\n", num_marked);

#ifdef DEBUG
  int seen_types[LAST_OBJ_TYPE] = {0};
#endif

  // go through each heap allocated object and gc them
  // not really the best, but it would be easy to improve
  for (size_t i = 0; i < gc_global_data.nodes.length; i++) {
    struct obj **ptr = vector_gc_heap_nodes_index_ptr(&gc_global_data.nodes, i);
    struct obj *obj = *ptr;

    if (obj == NULL) {
      continue;
    }

#ifdef DEBUG
    seen_types[obj->tag - 1]++;
#endif

    if (obj->mark == WHITE) {
      // free it, should this be done if the object is on the stack?
      if (DEBUG_ONLY(obj->on_stack)) {
        DEBUG_ONLY(RUNTIME_ERROR(
            "Object (%p, tag: %d, %s) was on the stack during a major GC!",
            (void *)obj, obj->tag, obj->last_touched_by));
      }

      // execute this object's free function
      gc_func_map[obj->tag].free(obj);

      free(obj);
      num_freed++;

      // set the pointer in the vector to null
      *ptr = NULL;
    } else if (DEBUG_ONLY(obj->mark == GREY)) {
      // this shouldn't happen, but just incase
      RUNTIME_ERROR("Object was marked grey at time of major GC!");
    } else {
      // reset marker now
      obj->mark = WHITE;
    }
  }

  DEBUG_FPRINTF(stderr, "freed %zu objects\n", num_freed);
  DEBUG_FPRINTF(stderr, "size of heap nodes: %zu\n",
                gc_global_data.nodes.length);

#ifdef DEBUG
  for (int i = 0; i < LAST_OBJ_TYPE; i++) {
    printf("tag %d seen %d times\n", i + 1, seen_types[i]);
  }
#endif

  gc_heap_maintain();
}

void gc_init(void) { gc_global_data.nodes = vector_gc_heap_nodes_new(100); }

// wrapped malloc that adds allocated stuff to the bookkeeper
void *gc_malloc(size_t size) {
  void *ptr = malloc(size);

  vector_gc_heap_nodes_push(&gc_global_data.nodes, ptr);
  return ptr;
}

void gc_heap_maintain(void) {
  size_t last_i = 0;
  size_t original_len = gc_global_data.nodes.length;
  for (size_t i = 0; i < gc_global_data.nodes.length; i++) {
    struct obj *obj = vector_gc_heap_nodes_index(&gc_global_data.nodes, i);
    if (obj != NULL) {
      vector_gc_heap_nodes_set(&gc_global_data.nodes, obj, last_i++);
    }
  }

  gc_global_data.nodes.length = last_i;
  if (last_i && (original_len / last_i) > 2)
    vector_gc_heap_nodes_shrink_to_fit(&gc_global_data.nodes);
}
