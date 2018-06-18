#include <stdbool.h>
#include <string.h>

#include "base.h"
#include "vec.h"
#include "queue.h"
#include "tree.h"
#include "gc.h"


MAKE_VECTOR(size_t, size_t)
MAKE_QUEUE(struct object *, gc_grey_nodes)
MAKE_QUEUE(struct ptr_toupdate_pair, ptr_toupdate_pair)


struct gc_funcs gc_func_map[] = {
    [CLOSURE] = (struct gc_funcs){
        .toheap = toheap_closure,
        .mark = mark_closure,
        .free = gc_free_noop
    },
    [ENV] = (struct gc_funcs){
        .toheap = toheap_env,
        .mark = mark_env,
        .free = free_env,
    },
};


// This does nothing, the gc will call free() on the object if it was heap allocated
void gc_free_noop(struct object *obj) {
    (void)obj;
}


static bool maybe_mark_grey(struct object *obj) {
    switch (obj->mark) {
        case BLACK:
        case GREY:
            return false;
        case WHITE:
            obj->mark = GREY;
            return true;
    }
}


struct object *toheap_closure(struct object *obj, struct gc_context *ctx) {
    struct closure *clos = (struct closure *)obj;

    if (!obj->on_stack) {
        return obj;
    }

    struct closure *heap_clos = malloc(sizeof(struct closure));
    memcpy(heap_clos, obj, sizeof(struct closure));

    queue_ptr_toupdate_pair_enqueue(&ctx->pointers_toupdate,
                                    (struct ptr_toupdate_pair){
                                        (struct object **)&heap_clos->env,
                                        (struct object *)heap_clos->env});

    return (struct object *)heap_clos;
}


// Someday we should optimise this idk
//
// Linear scan through an array of size_t
static bool linear_check_env_id(size_t *var_ids, size_t var_id, size_t num_ids) {
    for (size_t i = 0; i < num_ids; i++) {
        if (var_ids[i] == var_id) {
            return true;
        }
    }
    return false;
}

void mark_closure(struct object *obj, struct gc_context *ctx) {
    struct closure *clos = (struct closure *)obj;

    obj->mark = BLACK;

    struct env_table_entry env_table = global_env_table[clos->env_id];

    struct vector_size_t visited_env_ids = vector_size_t_new(env_table.num_ids);

    struct env_elem *env_head = clos->env;
    while (env_head) {
        if (!linear_check_env_id(env_table.var_ids, env_head->ident_id, env_table.num_ids)) {
            env_head = env_head->prev;
            continue;
        }
        if (!linear_check_env_id(visited_env_ids.data, env_head->ident_id, visited_env_ids.length)) {
            env_head = env_head->prev;
            continue;
        }

        env_head->base.mark = BLACK;
    }

    if (maybe_mark_grey(&clos->env->base)) {
        queue_gc_grey_nodes_enqueue(&ctx->grey_nodes, (struct object *)clos->env);
    }
}


struct object *toheap_env(struct object *obj, struct gc_context *ctx) {
    if (!obj->on_stack) {
        return obj;
    }

    struct env_elem *heap_env = malloc(sizeof(struct env_elem));
    memcpy(heap_env, obj, sizeof(struct env_elem));

    queue_ptr_toupdate_pair_enqueue(&ctx->pointers_toupdate,
                                    (struct ptr_toupdate_pair){
                                        (struct object **)&heap_env->val,
                                        (struct object *)heap_env->val});

    // TODO: update these for the vector
    for (size_t i=0; i<heap_env->nexts.length; i++) {
        struct env_elem **env_ptr = vector_env_elem_nexts_index_ptr(&heap_env->nexts, i);
        queue_ptr_toupdate_pair_enqueue(&ctx->pointers_toupdate, (struct ptr_toupdate_pair){(struct object **)env_ptr, (struct object *)*env_ptr});
    }
    return (struct object *)heap_env;
}


void mark_env(struct object *obj, struct gc_context *ctx) {
    struct env_elem *env = (struct env_elem *)obj;

    obj->mark = BLACK;

    for (size_t i=0; i < env->nexts.length; i++) {
        struct object *env_ptr = (struct object *)vector_env_elem_nexts_index(&env->nexts, i);
        if (maybe_mark_grey(env_ptr)) {
            queue_gc_grey_nodes_enqueue(&ctx->grey_nodes, env_ptr);
        }
    }
    if (maybe_mark_grey(env->val)) {
        queue_gc_grey_nodes_enqueue(&ctx->grey_nodes, env->val);
    }
}


void free_env(struct object *obj) {
    struct env_elem *env = (struct env_elem *)obj;

    // a -> b -> c
    //       \-> d
    // assuming we're freeing b, we need to add the pointers to c and d to a, and make c and d reference a
    //
    // a -> c
    //  \-> d

    for (size_t i=0; i < env->nexts.length; i++) {
        struct env_elem *child_ptr = vector_env_elem_nexts_index(&env->nexts, i);

        // make the node that would be a now reference each of b's children
        vector_env_elem_nexts_push(&env->prev->nexts, child_ptr);

        // make each of b's children reference a
        child_ptr->prev = env->prev;
    }

    vector_env_elem_nexts_free(&env->nexts);
}


struct gc_context gc_make_context(void) {
    return (struct gc_context){
        .grey_nodes = queue_gc_grey_nodes_new(10),
        .pointers_toupdate = queue_ptr_toupdate_pair_new(10),
        .updated_pointers = ptr_bst_new(),
    };
}

void gc_free_context(struct gc_context *ctx) {
    queue_gc_grey_nodes_free(&ctx->grey_nodes);
    queue_ptr_toupdate_pair_free(&ctx->pointers_toupdate);
    ptr_bst_free(&ctx->updated_pointers);
}


static void gc_mark_obj(struct gc_context *ctx, struct object *obj) {
    obj->mark = BLACK;
    gc_func_map[obj->tag].mark(obj, ctx);
}


// Moves all live objects on the stack over to the heap
struct object *gc_toheap(struct gc_context *ctx, struct object *obj) {
    // if we've already copied this object,
    // we know that anything it points to must also be sorted
    struct object *maybe_copied = ptr_bst_get(&ctx->updated_pointers, obj);
    if (maybe_copied != NULL) {
        return maybe_copied;
    }

    struct object *new_obj = gc_func_map[obj->tag].toheap(obj, ctx);

    // Add it to the updated tree
    // Even if it was on the heap already we still insert
    // since we then won't process child objects further
    struct ptr_pair pair = {.old = obj, .new = new_obj};
    ptr_bst_insert(&ctx->updated_pointers, pair);

    return new_obj;
}


// The minor gc, moves all stack objects to the heap
void gc_minor(struct gc_context *ctx, struct thunk *thnk) {

    // initially mark the closure and it's arguments to be applied
    thnk->closr = (struct closure *)gc_toheap(ctx, (struct object *)thnk->closr);

    switch (thnk->closr->size) {
        case CLOSURE_ONE:
            thnk->one.rand = gc_toheap(ctx, thnk->one.rand);
            break;
        case CLOSURE_TWO:
            thnk->two.rand = gc_toheap(ctx, thnk->two.rand);
            thnk->two.cont = gc_toheap(ctx, thnk->two.cont);
            break;
    }

    // work through each pointer that needs to be updated
    while (queue_ptr_toupdate_pair_len(&ctx->pointers_toupdate) > 0) {
        struct ptr_toupdate_pair to_update = queue_ptr_toupdate_pair_dequeue(&ctx->pointers_toupdate);
        struct object *maybe_copied = ptr_bst_get(&ctx->updated_pointers, to_update.on_stack);
        if (maybe_copied != NULL) {
            // we've already updated this pointer, just copy it
            *to_update.toupdate = maybe_copied;
        } else {
            // we haven't seen this yet, perform a copy and update
            struct object *on_heap = gc_toheap(ctx, to_update.on_stack);
            ptr_bst_insert(&ctx->updated_pointers, (struct ptr_pair){to_update.on_stack, on_heap});
            *to_update.toupdate = on_heap;
        }
    }
}


// The major gc, collects objects on the heap
// TODO: decide if this will fuck up if used when objects are still on the stack
void gc_major(struct gc_context *ctx, struct thunk *thnk) {
    gc_mark_obj(ctx, &thnk->closr->base);

    switch (thnk->closr->size) {
        case CLOSURE_ONE:
            gc_mark_obj(ctx, thnk->one.rand);
            break;
        case CLOSURE_TWO:
            gc_mark_obj(ctx, thnk->two.rand);
            gc_mark_obj(ctx, thnk->two.cont);
            break;
    }

    while (queue_gc_grey_nodes_len(&ctx->grey_nodes) > 0) {
        struct object *next_obj = queue_gc_grey_nodes_dequeue(&ctx->grey_nodes);
        gc_mark_obj(ctx, next_obj);
    }

}
