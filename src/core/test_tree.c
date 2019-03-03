#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#include "tree.h"

int main() {
  struct ptr_bst tree = ptr_bst_new();

  ptr_bst_insert(&tree, (struct ptr_pair){(void *)0, (void *)1});
  ptr_bst_insert(&tree, (struct ptr_pair){(void *)1, (void *)2});
  ptr_bst_insert(&tree, (struct ptr_pair){(void *)4, (void *)3});
  ptr_bst_insert(&tree, (struct ptr_pair){(void *)3, (void *)4});

  assert(ptr_bst_get(&tree, (void *)3) == (void *)4);
  assert(ptr_bst_get(&tree, (void *)0) == (void *)1);
  assert(ptr_bst_get(&tree, (void *)1) == (void *)2);
  assert(ptr_bst_get(&tree, (void *)4) == (void *)3);

  ptr_bst_clear(&tree);

  for (size_t i = 0; i < 100; i++) {
    ptr_bst_insert(&tree, (struct ptr_pair){(void *)i, (void *)(100 - i)});
  }

  for (size_t i = 0; i < 100; i++) {
    assert((void *)ptr_bst_get(&tree, (void *)i) == (void *)(100 - i));
  }

  ptr_bst_free(&tree);

  puts("yeah we're good (test_tree)");
}
