#include <string.h>

#include "base.h"
#include "gc.h"

struct gc_funcs gc_func_map[] = {
    [CLOSURE] = (struct gc_funcs){
        .toheap = toheap_closure,
        .mark = mark_closure
    }
};


struct object *toheap_closure(struct object *obj) {
    struct closure *heap_clos = malloc(sizeof(struct closure));
    memcpy(heap_clos, obj, sizeof(struct closure));
    return (struct object *)heap_clos;
}

void mark_closure(struct object *obj, struct gc_context *ctx) {
    // TODO: get environment marking working
    struct closure *clos = (struct closure *)obj;
    // env_mark(clos->env, ctx);
}
