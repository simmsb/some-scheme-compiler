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

DEFINE_VECTOR(struct ptr_pair_bst, ptr_pair_bst)

struct ptr_bst {
  struct vector_ptr_pair_bst vec;
};

struct ptr_bst ptr_bst_new(void);
void ptr_bst_insert(struct ptr_bst *, struct ptr_pair);
void *ptr_bst_get(struct ptr_bst *, void *);
void ptr_bst_free(struct ptr_bst *);
void ptr_bst_clear(struct ptr_bst *);

#endif /* SOMESCHEME_TREE_H */
