#include <stdlib.h>
#include <stdint.h>

#include "vec.h"
#include "tree.h"

MAKE_VECTOR(struct ptr_pair_bst, ptr_pair_bst)

struct ptr_bst ptr_bst_new(void) {
    struct vector_ptr_pair_bst vec = vector_ptr_pair_bst_new(10);
    return (struct ptr_bst){vec};
}

void ptr_bst_insert(struct ptr_bst *bst, struct ptr_pair pair) {
    struct ptr_pair_bst elem = {pair, -1, -1};
    int inserted_idx = (int)vector_ptr_pair_bst_push(&bst->vec, elem);
    if (bst->vec.length != 0) {
        // root is the first in the tree
        int idx = 0;
        struct ptr_pair_bst *cur;
        for (;;) {
            // get the current node
            cur = vector_ptr_pair_bst_index_ptr(&bst->vec, (size_t)idx);
            // you 'will' let me compare pointers
            // do the traversal thing
            if ((uintptr_t)cur->ptrs.old == (uintptr_t)pair.old) {
                // This is already inserted for some reason?
                return;
            } else if ((uintptr_t)cur->ptrs.old < (uintptr_t)pair.old) {
                idx = cur->right_idx;
                if (idx == -1) {
                    cur->right_idx = inserted_idx;
                    break;
                }
            } else {
                idx = cur->left_idx;
                if (idx == -1) {
                    cur->left_idx = inserted_idx;
                    break;
                }
            }
        }
    }
    // What am I doing?
}


void *ptr_bst_get(struct ptr_bst *bst, void *ptr) {
    uintptr_t ptr_int = (uintptr_t)ptr;
    if (bst->vec.length == 0) {
        return NULL;
    } else {
        int idx = 0;
        struct ptr_pair_bst *cur;
        while (idx != -1) {
            cur = vector_ptr_pair_bst_index_ptr(&bst->vec, (size_t)idx);
            if ((uintptr_t)cur->ptrs.old == ptr_int) {
                return cur->ptrs.new;
            } else if ((uintptr_t)cur->ptrs.old < ptr_int) {
                idx = cur->right_idx;
            } else {
                idx = cur->left_idx;
            }
        }
        return NULL;
    }
}


void ptr_bst_free(struct ptr_bst bst) {
    vector_ptr_pair_bst_free(&bst.vec);
}

void ptr_bst_clear(struct ptr_bst *bst) {
    // to clear the tree, just set the vector to zero such
    // that the next set will write to index 0, or get will return NULL
    bst->vec.length = 0;
}
