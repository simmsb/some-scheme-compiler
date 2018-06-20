#ifndef SOMESCHEME_GC_H
#define SOMESCHEME_GC_H

#include "base.h"
#include "vec.h"
#include "tree.h"
#include "queue.h"

DEFINE_VECTOR(size_t, size_t)
DEFINE_VECTOR(struct object *, gc_heap_nodes)
DEFINE_QUEUE(struct object *, gc_grey_nodes)

struct ptr_toupdate_pair {
    struct object **toupdate;
    struct object *on_stack;
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
    struct ptr_bst updated_pointers;
};

struct gc_funcs {
    // Copies the object to the heap and updates
    // anything it points to to point to the heap
    // if the object is on the heap already this returns the same
    // pointer that was put in
    struct object *(* const toheap)(struct object *, struct gc_context *);

    // Marks an object and any child pointers
    // Stack objects are copied to the heap and the context updated
    void (* const mark)(struct object *, struct gc_context *);

    // Frees an object
    // Acts as the cleanup routine, the gc will decide whether to call free on
    // the object if it is on the stack or not
    void (* const free)(struct object *);
};


struct gc_data {
    struct vector_gc_heap_nodes nodes;
};


void gc_init(void);

void gc_free_noop(struct object *);

struct object *toheap_closure(struct object *, struct gc_context *);
void mark_closure(struct object *, struct gc_context *);

struct object *toheap_env(struct object *, struct gc_context *);
void mark_env(struct object *, struct gc_context *);
void free_env(struct object *);


struct gc_context gc_make_context(void);
void gc_free_context(struct gc_context *);
void gc_minor(struct gc_context *, struct thunk *);
void gc_major(struct gc_context *, struct thunk *);
struct object *gc_toheap(struct gc_context *, struct object *);

void gc_heap_maintain(void);
void *gc_malloc(size_t);

#endif // SOMESCHEME_GC_H
