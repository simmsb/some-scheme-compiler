#include <setjmp.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>

#include "base.h"

static bool stack_check(void);

static struct thunk *current_thunk;
static void *stack_initial;
static jmp_buf setjmp_env_buf;

extern struct env_table_entry env_table[];

#define ADD_ENV(IDENT, VAL, HEAD)                                              \
    do {                                                                       \
        struct env_elem *new_env = alloca(sizeof(struct env_elem));            \
        memcpy(new_env,                                                        \
               &(struct env_elem){.base = object_base_new(ENV),                \
                                  .ident_id = (IDENT),                         \
                                  .val = (VAL),                                \
                                  .next = (HEAD)},                             \
               sizeof(struct env_elem));                                       \
                                                                               \
        (HEAD) = new_env;                                                      \
    } while (0)

void call_closure_one(struct object *rator, size_t rand_id,
                      struct object *rand) {
    if (rator->tag != 0) {
        RUNTIME_ERROR("Called object was not a closure");
    }

    struct closure *closure = (struct closure *)rator;

    if (closure->size == CLOSURE_TWO) {
        RUNTIME_ERROR("Called a closure that takes two args with one arg");
    }

    ADD_ENV(rand_id, rand, closure->env);
    if (stack_check()) {
        closure->fn_1(rand, closure->env);
    } else {
        // TODO, move to our own gc allocator?
        struct thunk thnk = {
            .closr = closure,
            .one = {rand},
        };
        struct thunk *thnk_heap = malloc(sizeof(struct thunk));
        memcpy(thnk_heap, &thnk, sizeof(struct thunk));
        run_minor_gc(thnk_heap);
    }
}

void call_closure_two(struct object *rator, size_t rand_id, struct object *rand,
                      size_t cont_id, struct object *cont) {
    if (rator->tag != 0) {
        RUNTIME_ERROR("Called object was not a closure");
    }

    struct closure *closure = (struct closure *)rator;

    if (closure->size != CLOSURE_ONE) {
        RUNTIME_ERROR("Called a closure that takes two args with one arg");
    }

    ADD_ENV(rand_id, rand, closure->env);
    ADD_ENV(cont_id, cont, closure->env);

    if (stack_check()) {
        closure->fn_2(rand, cont, closure->env);
    } else {
        // TODO, move to our own gc allocator?
        struct thunk thnk = {
            .closr = closure, // copy the closure
            .two = {rand, cont},
        };
        struct thunk *thnk_heap = malloc(sizeof(struct thunk));
        memcpy(thnk_heap, &thnk, sizeof(struct thunk));
        run_minor_gc(thnk_heap);
    }
}

void halt_func(struct object *cont, struct env_elem *env) {
    (void)cont; // mmh
    (void)env;
    printf("Halt");
    exit(0);
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
    static size_t stack_buffer = 1024 * 32;
    return (uintptr_t)stack_ptr() >
           (uintptr_t)(stack_initial - get_stack_limit() - stack_buffer);
}

void scheme_start(struct thunk *initial_thunk) {
    stack_initial = stack_ptr();
    current_thunk = initial_thunk;

    // This is our trampoline, when we come back from a longjmp a different
    // current_thunk will be set and we will just trampoline into the new
    // thunk
    setjmp(setjmp_env_buf);

    if (!current_thunk->closr->size) {
        current_thunk->closr->fn_1(current_thunk->one.rand,
                                   current_thunk->closr->env);
    } else {
        current_thunk->closr->fn_2(current_thunk->two.rand,
                                   current_thunk->two.cont,
                                   current_thunk->closr->env);
    }

    RUNTIME_ERROR("Control flow returned from trampoline function.");
}

void run_minor_gc(struct thunk *thnk) {
    // Minor gc happens here
    // Peek into the thunk and gc all live objects
    // For each object type run it's relevant minor-gc routine
    // Where we pass an array of already collected objects
    // (a list of old pointer to new pointer tuples)
    // and each time we see an unseen before pointer we run the gc routine on it
    // and copy it to heap and add it to the vector of collected objects.
    // If we see an already copied pointer, just mutate it to the new one.
    // The end result should be that everything that was alive is now in the
    // heap. We ofcourse need to check if we have a pointer to heap memory or a
    // pointer to stack memory We should do this by having a table of pointers
    // that are on the heap. If we see a pointer that is not on the heap then it
    // must be a stack pointer (Don't do any pointer range magick) We should
    // also create a union of all live environment vars, done by adding each
    // var_ids array from the env_table for each closure we see, and also making
    // sure that we do a gc of each env object such that closures stored inside
    // env_vars have their environemnts preserved
}

struct object object_base_new(enum object_tag tag) {
    return (struct object){
        .tag = tag,
        .mark = WHITE,
        .on_stack = true,
    };
}
