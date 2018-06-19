#ifndef SOMESCHEME_VEC_H
#define SOMESCHEME_VEC_H

#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define RUNTIME_ERROR(S)                                                       \
    do {                                                                       \
        fprintf(stderr, "Runtime Error (%s:%d): %s\n", __func__, __LINE__,     \
                (S));                                                          \
        exit(1);                                                               \
    } while (0)

#define DEFINE_VECTOR(TYPE, TNAME)                                             \
    struct vector_##TNAME {                                                    \
        size_t cap;                                                            \
        size_t length;                                                         \
        TYPE *data;                                                            \
    };                                                                         \
    struct vector_##TNAME vector_##TNAME##_new(size_t);                        \
    TYPE vector_##TNAME##_pop(struct vector_##TNAME *);                        \
    size_t vector_##TNAME##_push(struct vector_##TNAME *, TYPE);               \
    TYPE vector_##TNAME##_index(struct vector_##TNAME *, size_t);              \
    void vector_##TNAME##_set(struct vector_##TNAME *, TYPE, size_t);          \
    TYPE *vector_##TNAME##_index_ptr(struct vector_##TNAME *, size_t);         \
    void vector_##TNAME##_shrink_to_fit(struct vector_##TNAME *);              \
    void vector_##TNAME##_free(struct vector_##TNAME *);                       \
    void vector_##TNAME##_remove(struct vector_##TNAME *, size_t);             \
    size_t vector_##TNAME##_indexof(struct vector_##TNAME *, TYPE);

#define MAKE_VECTOR(TYPE, TNAME)                                               \
    struct vector_##TNAME vector_##TNAME##_new(size_t initial) {               \
        TYPE *data = malloc(sizeof(TYPE) * initial);                           \
        return (struct vector_##TNAME){initial, 0, data};                      \
    }                                                                          \
    TYPE vector_##TNAME##_pop(struct vector_##TNAME *vec) {                    \
        if (vec->length == 0) {                                                \
            RUNTIME_ERROR("Popping from 0-length vector");                     \
        }                                                                      \
        TYPE elem = vec->data[vec->length - 1];                                \
        vec->length--;                                                         \
        return elem;                                                           \
    }                                                                          \
    size_t vector_##TNAME##_push(struct vector_##TNAME *vec, TYPE elem) {      \
        if (vec->length >= vec->cap) {                                         \
            size_t new_len = 1 + vec->cap + (vec->cap >> 2);                   \
            vec->data = realloc(vec->data, new_len * sizeof(TYPE));            \
            vec->cap = new_len;                                                \
        }                                                                      \
        size_t inserted_idx = vec->length;                                     \
        vec->data[vec->length++] = elem;                                       \
        return inserted_idx;                                                   \
    }                                                                          \
    TYPE vector_##TNAME##_index(struct vector_##TNAME *vec, size_t idx) {      \
        if (idx < 0 || idx >= vec->length) {                                   \
            RUNTIME_ERROR("Indexing vector out of bounds");                    \
        }                                                                      \
        return vec->data[idx];                                                 \
    }                                                                          \
    void vector_##TNAME##_set(struct vector_##TNAME *vec, TYPE elem,           \
                              size_t idx) {                                    \
        if (idx < 0 || idx >= vec->length) {                                   \
            RUNTIME_ERROR("Indexing vector out of bounds");                    \
        }                                                                      \
        vec->data[idx] = elem;                                                 \
    }                                                                          \
    TYPE *vector_##TNAME##_index_ptr(struct vector_##TNAME *vec, size_t idx) { \
        if (idx < 0 || idx >= vec->length) {                                   \
            RUNTIME_ERROR("Indexing vector out of bounds");                    \
        }                                                                      \
        return &vec->data[idx];                                                \
    }                                                                          \
    void vector_##TNAME##_shrink_to_fit(struct vector_##TNAME *vec) {          \
        vec->data = realloc(vec->data, vec->length * sizeof(TYPE));            \
        vec->cap = vec->length;                                                \
    }                                                                          \
    void vector_##TNAME##_free(struct vector_##TNAME *vec) {                   \
        free(vec->data);                                                       \
    }                                                                          \
    void vector_##TNAME##_remove(struct vector_##TNAME *vec, size_t idx) {     \
        if (idx < 0 || idx >= vec->length) {                                   \
            RUNTIME_ERROR("Indexing vector out of bounds");                    \
        }                                                                      \
        memmove(&vec->data[idx], &vec->data[idx + 1], vec->length - idx - 1);  \
        vec->length--;                                                         \
    }                                                                          \
    size_t vector_##TNAME##_indexof(struct vector_##TNAME *vec, TYPE val) {    \
        for (size_t i = 0; i < vec->length; i++) {                             \
            if (memcmp(&vec->data[i], &val, sizeof(TYPE)) == 0) {              \
                return i;                                                      \
            }                                                                  \
        }                                                                      \
        return vec->length;                                                    \
    }

#endif /* SOMESCHEME_VEC_H */
