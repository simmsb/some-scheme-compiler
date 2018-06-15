#ifndef SOMESCHEME_GC_H
#define SOMESCHEME_GC_H

#include "base.h"
#include "vec.h"
#include "tree.h"

// Vector for variables that are active
DEFINE_VECTOR(size_t, gc_env_active)

enum mark_type {
    WHITE = 0,
    GREY,
    BLACK
};

struct gc_context {
    struct vector_gc_env_active active_vars;
    struct ptr_bst updated_pointers;
};

struct gc_funcs {
    // Copies the object to the heap, does nothing to the object
    struct object *(* const toheap)(struct object *);

    // Marks an object and any child pointers
    // Stack objects are copied to the heap and the context updated
    void (* const mark)(struct object *, struct gc_context *);
};


// array of gc_funcs for each object type
extern struct gc_funcs gc_func_map[];


struct object *toheap_closure(struct object *);
void mark_closure(struct object *, struct gc_context *);


#endif // SOMESCHEME_GC_H
