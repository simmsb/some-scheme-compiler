#include <string.h>

#include "base.h"
#include "vec.h"
#include "queue.h"
#include "tree.h"
#include "gc.h"


MAKE_VECTOR(size_t, gc_env_active)
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
        .free = gc_free_noop,
    },
};


void gc_free_noop(struct object *obj) {
    (void)obj;
}


struct object *toheap_closure(struct object *obj, struct gc_context *ctx) {
    struct closure *clos = (struct closure *)obj;

    if (!obj->on_stack) {
        return obj;
    }

    struct closure *heap_clos = malloc(sizeof(struct closure));
    memcpy(heap_clos, obj, sizeof(struct closure));

    queue_ptr_toupdate_pair_enqueue(&ctx->pointers_toupdate, (struct ptr_toupdate_pair){(struct object **)&heap_clos->env, (struct object *)heap_clos->env});

    return (struct object *)heap_clos;
}

void mark_closure(struct object *obj, struct gc_context *ctx) {
    // TODO: get environment marking working
    struct closure *clos = (struct closure *)obj;

    obj->mark = BLACK;

    queue_gc_grey_nodes_enqueue(&ctx->grey_nodes, (struct object *)clos->env);
}


struct object *toheap_env(struct object *obj, struct gc_context *ctx) {
    if (!obj->on_stack) {
        return obj;
    }

    struct env_elem *heap_env = malloc(sizeof(struct env_elem));
    memcpy(heap_env, obj, sizeof(struct env_elem));

    queue_ptr_toupdate_pair_enqueue(&ctx->pointers_toupdate, (struct ptr_toupdate_pair){(struct object **)&heap_env->val, (struct object *)heap_env->val});
    queue_ptr_toupdate_pair_enqueue(&ctx->pointers_toupdate, (struct ptr_toupdate_pair){(struct object **)&heap_env->next, (struct object *)heap_env->next});
    return (struct object *)heap_env;
}


void mark_env(struct object *obj, struct gc_context *ctx) {
    struct env_elem *env = (struct env_elem *)obj;

    obj->mark = BLACK;

    queue_gc_grey_nodes_enqueue(&ctx->grey_nodes, (struct object *)env->next);
    queue_gc_grey_nodes_enqueue(&ctx->grey_nodes, env->val);
}


struct gc_context gc_make_context(void) {
    return (struct gc_context){
        .active_vars = vector_gc_env_active_new(10),
        .grey_nodes = queue_gc_grey_nodes_new(10),
        .pointers_toupdate = queue_ptr_toupdate_pair_new(10),
        .updated_pointers = ptr_bst_new(),
    };
}


static void gc_mark_obj(struct gc_context *ctx, struct object *obj) {
    obj->mark = BLACK;
    gc_func_map[obj->tag].mark(obj, ctx);
}


// Moves all live objects on the stack over to the heap
struct object *gc_toheap(struct gc_context *ctx, struct object *obj) {
    struct object *maybe_copied = ptr_bst_get(&ctx->updated_pointers, obj);
    if (maybe_copied != NULL) {
        return maybe_copied;
    }

    struct object *new_obj = gc_func_map[obj->tag].toheap(obj, ctx);

    if (obj->on_stack) {
        struct ptr_pair pair = {.old = obj, .new = new_obj};

        ptr_bst_insert(&ctx->updated_pointers, pair);
    }

    return new_obj;
}

void gc_run(struct gc_context *ctx, struct thunk *thnk) {
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
