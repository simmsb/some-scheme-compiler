#ifndef SOMESCHEME_GC_H
#define SOMESCHEME_GC_H

#include "base.h"
#include "queue.h"
#include "hash_table.h"
#include "vec.h"

DEFINE_VECTOR(size_t, size_t);
DEFINE_VECTOR(struct obj *, gc_heap_nodes);
DEFINE_QUEUE(struct obj *, gc_grey_nodes);
DEFINE_HASH(struct obj *, ptr_map);

struct ptr_toupdate_pair {
  struct obj **toupdate;
  struct obj *on_stack;
};

DEFINE_QUEUE(struct ptr_toupdate_pair, ptr_toupdate_pair)

struct gc_context {
  // nodes that are marked grey
  struct queue_gc_grey_nodes grey_nodes;

  // pointers that need to be updated when
  // another pointer has been moved to the heap
  // pair is (pointer_to_update, stack_pointer)
  struct queue_ptr_toupdate_pair pointers_toupdate;

  // pointers that have been updated to the heap
  // pair is (stack_pointer, heap_pointer)
  struct hash_table_ptr_map *updated_pointers;
};

struct gc_funcs {
  // Copies the object to the heap and updates
  // anything it points to to point to the heap
  // if the object is on the heap already this returns the same
  // pointer that was put in
  struct obj *(*const toheap)(struct obj *, struct gc_context *);

  // Marks an object and any child pointers
  // Stack objects are copied to the heap and the context updated
  void (*const mark)(struct obj *, struct gc_context *);

  // Frees an object
  // Acts as the cleanup routine, the gc will decide whether to call free on
  // the object if it is on the stack or not
  void (*const free)(struct obj *);
};

struct gc_data {
  struct vector_gc_heap_nodes nodes;
};

void gc_init(void);

void gc_free_noop(struct obj *);
void gc_mark_noop(struct obj *, struct gc_context *);

struct obj *toheap_closure(struct obj *, struct gc_context *);
void mark_closure(struct obj *, struct gc_context *);


struct obj *toheap_env(struct obj *, struct gc_context *);
void mark_env(struct obj *, struct gc_context *);

struct obj *toheap_int_obj(struct obj *, struct gc_context *);

struct obj *toheap_string_obj(struct obj *, struct gc_context *);

struct obj *toheap_cell(struct obj *, struct gc_context *);
void mark_cell(struct obj *, struct gc_context *);

struct gc_context gc_make_context(void);

void gc_free_context(struct gc_context *);
void gc_minor(struct gc_context *, struct thunk *);
void gc_major(struct gc_context *, struct thunk *);

struct obj *gc_toheap(struct gc_context *, struct obj *);
void gc_mark_obj(struct gc_context *, struct obj *);

void gc_heap_maintain(void);
void *gc_malloc(size_t);

#endif // SOMESCHEME_GC_H
