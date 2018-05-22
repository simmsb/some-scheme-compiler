#include <stdlib.h>

struct env_elem {
    const char *ident;     // owned by env_elem
    struct object *val;    // shared
    struct env_elem *next; // owned by env_elem
};

struct closure {
    void (*fn)(struct object *, struct closure *);
    struct env_elem *env;
};

#define ADD_ENV(IDENT, VAL, HEAD)                                              \
    do {                                                                       \
        struct env_elem *new_env = alloca(sizeof(struct env_elem));            \
                                                                               \
        size_t ident_len = strlen(IDENT) + 1;                                  \
                                                                               \
        new_env->indent = alloca(ident_len);                                   \
        strcpy(new_env->ident, IDENT);                                         \
                                                                               \
        new_env->val = VAL;                                                    \
        new_env->next = HEAD;                                                  \
                                                                               \
        HEAD = new_env;                                                        \
    } while (0)
