#ifndef SOMESCHEME_TREE_H
#define SOMESCHEME_TREE_H

#include "vec.h"

struct ptr_pair {
    void *old;
    void *new;
};

struct ptr_pair_bst {
    struct ptr_pair ptrs;
    int left_idx;
    int right_idx;
};

MAKE_VECTOR(struct ptr_pair_bst, ptr_pair_bst)

struct ptr_bst {
    vector_ptr_pair_bst vec;
};


struct ptr_bst ptr_bst_new(void) {
    auto vec = struct vector_ptr_pair_bst_new(10);
    return (struct ptr_bst){vec};
}

void ptr_bst_insert(struct ptr_bst *bst, struct ptr_pair pair) {
    struct ptr_pair_bst elem = {pair, -1, -1};
    size_t inserted_idx = vector_ptr_pair_bst_push(&bst->vec, elem);
    if (bst->vec.length != 0) {
        // root is the first in the tree
        int idx = 0;
        struct ptr_pair_bst *cur;
        for (;;) {
            cur = vector_ptr_pair_bst_index_ptr(&bst->vec, idx);
            // you 'will' let me compare pointers
            if ((size_t)cur->ptrs.old < (size_t)pair.old) {
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

#endif /* SOMESCHEME_TREE_H */
