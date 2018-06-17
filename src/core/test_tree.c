#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>
#include <assert.h>

#include "tree.h"

int main() {
    struct ptr_bst tree = ptr_bst_new();

    ptr_bst_insert(&tree, (struct ptr_pair){ (void *)0, (void *)1 });
    ptr_bst_insert(&tree, (struct ptr_pair){ (void *)1, (void *)2 });
    ptr_bst_insert(&tree, (struct ptr_pair){ (void *)4, (void *)3 });
    ptr_bst_insert(&tree, (struct ptr_pair){ (void *)3, (void *)4 });


    printf("idx: %d val: %ld\n", 3, (intptr_t)(void *)ptr_bst_get(&tree, (void *)3));
    printf("idx: %d val: %ld\n", 0, (intptr_t)(void *)ptr_bst_get(&tree, (void *)0));
    printf("idx: %d val: %ld\n", 1, (intptr_t)(void *)ptr_bst_get(&tree, (void *)1));
    printf("idx: %d val: %ld\n", 4, (intptr_t)(void *)ptr_bst_get(&tree, (void *)4));

    assert(ptr_bst_get(&tree, (void *)3) == (void *)4);
    assert(ptr_bst_get(&tree, (void *)0) == (void *)1);
    assert(ptr_bst_get(&tree, (void *)1) == (void *)2);
    assert(ptr_bst_get(&tree, (void *)4) == (void *)3);

    for (size_t i=0; i<100; i++) {
        ptr_bst_insert(&tree, (struct ptr_pair){ (void *)i, (void *)(100 - i) });
    }

    for (size_t i=0; i<100; i++) {
        printf("idx: %ld val: %ld\n", i, (intptr_t)(void *)ptr_bst_get(&tree, (void *)i));
    }

    ptr_bst_free(tree);
}
