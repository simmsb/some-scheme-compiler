#ifndef __HASH_H_
#define __HASH_H_

// A hash table implementation using robin hood hashing

#include <stdbool.h>
#include <stdint.h>

#include "bit_array.h"
#include "common.h"

static const uint32_t hash_table_initial_cap = 64;
static const uint8_t hash_table_load_factor_to_grow = 90;

#define HASH_TABLE_ITER(NAME, KEY_NAME, VAL_NAME, TABLE, ...)                  \
  for (size_t hash_table_##NAME##_iter_idx = 0;                                \
       hash_table_##NAME##_iter_idx < (TABLE)->cap;                            \
       hash_table_##NAME##_iter_idx++) {                                       \
    struct hash_table_##NAME##_elem *hash_table_##NAME##_iter_e =              \
        &(TABLE)->elems[hash_table_##NAME##_iter_idx];                         \
    if (hash_table_##NAME##_iter_e->hash &&                                    \
        !hash_table_##NAME##_is_entry_deleted((TABLE),                         \
                                              hash_table_##NAME##_iter_idx)) { \
      size_t KEY_NAME = hash_table_##NAME##_iter_e->key;                       \
      typeof(hash_table_##NAME##_iter_e->val) *VAL_NAME =                      \
          &hash_table_##NAME##_iter_e->val;                                    \
      { __VA_ARGS__ }                                                          \
    }                                                                          \
  }

#define DEFINE_HASH(VALTYPE, NAME)                                             \
  struct hash_table_##NAME##_elem {                                            \
    size_t hash;                                                               \
    size_t key;                                                                \
    VALTYPE val;                                                               \
  };                                                                           \
                                                                               \
  struct hash_table_##NAME {                                                   \
    struct hash_table_##NAME##_elem *elems;                                    \
    uint8_t *deleted;                                                          \
    uint32_t num_elems;                                                        \
    uint32_t cap;                                                              \
    size_t mask;                                                               \
    uint resize_thresh;                                                        \
  };                                                                           \
  struct hash_table_##NAME *hash_table_##NAME##_new();                         \
  void hash_table_##NAME##_free(struct hash_table_##NAME *table);              \
  void hash_table_##NAME##_insert(struct hash_table_##NAME *table, size_t k,   \
                                  VALTYPE v);                                  \
  VALTYPE *hash_table_##NAME##_lookup(struct hash_table_##NAME *table,         \
                                      size_t k);                               \
  bool hash_table_##NAME##_delete(struct hash_table_##NAME *table, size_t k);  \
  bool hash_table_##NAME##_is_entry_deleted(struct hash_table_##NAME *table,   \
                                            size_t idx);                       \
  void hash_table_##NAME##_clear(struct hash_table_##NAME *table);

#define MAKE_HASH(VALTYPE, NAME)                                               \
  bool hash_table_##NAME##_is_entry_deleted(struct hash_table_##NAME *table,   \
                                            size_t idx) {                      \
    return get_bit_in_bitarray(table->deleted, idx);                           \
  }                                                                            \
                                                                               \
  static void hash_table_##NAME##__mark_deleted(                               \
      struct hash_table_##NAME *table, size_t idx) {                           \
    set_bit_in_bitarray(table->deleted, idx, true);                            \
  }                                                                            \
                                                                               \
  static void hash_table_##NAME##__reset_deleted(                              \
      struct hash_table_##NAME *table, size_t idx) {                           \
    set_bit_in_bitarray(table->deleted, idx, false);                           \
  }                                                                            \
                                                                               \
  static size_t hash_table_##NAME##__hash_fun(size_t k) {                      \
    k = ((k >> 30) ^ k) * 0xbf58476d1ce4e5b9;                                  \
    k = ((k >> 27) ^ k) * 0xbf58476d1ce4e5b9;                                  \
    k = (k >> 31) ^ k;                                                         \
                                                                               \
    return k;                                                                  \
  }                                                                            \
                                                                               \
  static size_t hash_table_##NAME##__fix_hash(size_t h) {                      \
    if (h) {                                                                   \
      return h;                                                                \
    }                                                                          \
                                                                               \
    return 1;                                                                  \
  }                                                                            \
                                                                               \
  static size_t hash_table_##NAME##__hash_idx(struct hash_table_##NAME *table, \
                                              size_t hash) {                   \
                                                                               \
    return hash & table->mask;                                                 \
  }                                                                            \
                                                                               \
  static size_t hash_table_##NAME##__max_probes(                               \
      struct hash_table_##NAME *table, size_t hash, size_t idx) {              \
    return (table->cap + idx - hash_table_##NAME##__hash_idx(table, hash)) &   \
           table->mask;                                                        \
  }                                                                            \
                                                                               \
  static void hash_table_##NAME##__insert(struct hash_table_##NAME *table,     \
                                          struct hash_table_##NAME##_elem e) { \
    size_t idx = hash_table_##NAME##__hash_idx(table, e.hash);                 \
    /* printf("%u\n", idx); */                                                 \
    size_t to_insert_elem_probes = 0;                                          \
                                                                               \
    for (;;) {                                                                 \
      /* fast case, element where we want to insert is empty */                \
      if (!table->elems[idx].hash) {                                           \
        table->elems[idx] = e;                                                 \
                                                                               \
        return;                                                                \
      }                                                                        \
                                                                               \
      /* fprintf(stderr, "elem at: %d, k: %d, deleted?: %d\n", idx,            \
       * table->elems[idx].key, */                                             \
      /* hash_table_##NAME##_is_entry_deleted(table, idx));  */                \
                                                                               \
      size_t current_elem_probes =                                             \
          hash_table_##NAME##__max_probes(table, table->elems[idx].hash, idx); \
                                                                               \
      /* the element is deleted, just replace it  */                           \
      if (hash_table_##NAME##_is_entry_deleted(table, idx)) {                  \
                                                                               \
        /* undelete  */                                                        \
        hash_table_##NAME##__reset_deleted(table, idx);                        \
                                                                               \
        table->elems[idx] = e;                                                 \
                                                                               \
        return;                                                                \
      }                                                                        \
      /* if we're here, the element was occupied or deleted  */                \
      /* steal from the rich, give to the poor  */                             \
      if (current_elem_probes < to_insert_elem_probes) {                       \
        /* element wasn't deleted, swap element to insert with it and continue \
         */                                                                    \
        SWAP(e, table->elems[idx]);                                            \
        to_insert_elem_probes = current_elem_probes;                           \
      }                                                                        \
                                                                               \
      idx++;                                                                   \
      idx &= table->mask;                                                      \
      to_insert_elem_probes++;                                                 \
                                                                               \
      /* printf("idx: %d, number probes: %d\n", idx, to_insert_elem_probes);   \
       */                                                                      \
    }                                                                          \
  }                                                                            \
                                                                               \
  static int64_t hash_table_##NAME##__lookup(struct hash_table_##NAME *table,  \
                                             size_t k) {                       \
    size_t hash =                                                              \
        hash_table_##NAME##__fix_hash(hash_table_##NAME##__hash_fun(k));       \
    size_t idx = hash_table_##NAME##__hash_idx(table, hash);                   \
                                                                               \
    size_t num_probes = 0;                                                     \
                                                                               \
    for (;;) {                                                                 \
      size_t current_hash = table->elems[idx].hash;                            \
                                                                               \
      /* if the entry is empty and not deleted, nothing is here  */            \
      if (!current_hash) {                                                     \
        return -1;                                                             \
      }                                                                        \
                                                                               \
      /* if we've proved enough times to check every possible entry, nothing   \
       * is  */                                                                \
      /* here  */                                                              \
      if (num_probes >                                                         \
          hash_table_##NAME##__max_probes(table, current_hash, idx)) {         \
        return -1;                                                             \
      }                                                                        \
                                                                               \
      /* current element isn't deleted, and both the hash and keys match  */   \
      if (!hash_table_##NAME##_is_entry_deleted(table, idx) &&                 \
          current_hash == hash && table->elems[idx].key == k) {                \
        return idx;                                                            \
      }                                                                        \
                                                                               \
      idx++;                                                                   \
      idx &= table->mask;                                                      \
      num_probes++;                                                            \
    }                                                                          \
  }                                                                            \
                                                                               \
  static void hash_table_##NAME##__construct(struct hash_table_##NAME *table,  \
                                             uint32_t initial_capacity) {      \
    table->elems =                                                             \
        calloc(initial_capacity, sizeof(struct hash_table_##NAME##_elem));     \
    table->deleted = bit_array_new(initial_capacity);                          \
    table->num_elems = 0;                                                      \
    table->cap = initial_capacity;                                             \
    table->mask = initial_capacity - 1;                                        \
    table->resize_thresh =                                                     \
        (initial_capacity * hash_table_load_factor_to_grow) / 100;             \
  }                                                                            \
                                                                               \
  static void hash_table_##NAME##__grow(struct hash_table_##NAME *table) {     \
    struct hash_table_##NAME new_table;                                        \
    hash_table_##NAME##__construct(&new_table, table->cap * 2);                \
                                                                               \
    new_table.num_elems = table->num_elems;                                    \
                                                                               \
    for (uint32_t i = 0; i < table->cap; i++) {                                \
      struct hash_table_##NAME##_elem e = table->elems[i];                     \
                                                                               \
      if (e.hash && !hash_table_##NAME##_is_entry_deleted(table, i)) {         \
        hash_table_##NAME##__insert(&new_table, e);                            \
      }                                                                        \
    }                                                                          \
                                                                               \
    hash_table_##NAME##_free(table);                                           \
    *table = new_table;                                                        \
  }                                                                            \
                                                                               \
  struct hash_table_##NAME *hash_table_##NAME##_new() {                        \
    struct hash_table_##NAME *table =                                          \
        malloc(sizeof(struct hash_table_##NAME));                              \
    hash_table_##NAME##__construct(table, hash_table_initial_cap);             \
    return table;                                                              \
  }                                                                            \
                                                                               \
  void hash_table_##NAME##_free(struct hash_table_##NAME *table) {             \
    free(table->elems);                                                        \
    free(table->deleted);                                                      \
  }                                                                            \
                                                                               \
  void hash_table_##NAME##_insert(struct hash_table_##NAME *table, size_t k,   \
                                  VALTYPE v) {                                 \
    size_t hash =                                                              \
        hash_table_##NAME##__fix_hash(hash_table_##NAME##__hash_fun(k));       \
                                                                               \
    /* printf("num_elems: %d, resize_thresh: %d\n", table->num_elems,          \
     * table->resize_thresh); */                                               \
                                                                               \
    table->num_elems++;                                                        \
                                                                               \
    if (table->num_elems >= table->resize_thresh) {                            \
      /* printf("growing table\n"); */                                         \
      hash_table_##NAME##__grow(table);                                        \
    }                                                                          \
                                                                               \
    hash_table_##NAME##__insert(                                               \
        table, (struct hash_table_##NAME##_elem){hash, k, v});                 \
  }                                                                            \
                                                                               \
  VALTYPE *hash_table_##NAME##_lookup(struct hash_table_##NAME *table,         \
                                      size_t k) {                              \
    int64_t idx = hash_table_##NAME##__lookup(table, k);                       \
                                                                               \
    if (idx < 0) {                                                             \
      return NULL;                                                             \
    }                                                                          \
    return &table->elems[idx].val;                                             \
  }                                                                            \
                                                                               \
  bool hash_table_##NAME##_delete(struct hash_table_##NAME *table, size_t k) { \
    int64_t idx = hash_table_##NAME##__lookup(table, k);                       \
                                                                               \
    if (idx < 0) {                                                             \
      return false;                                                            \
    }                                                                          \
                                                                               \
    hash_table_##NAME##__mark_deleted(table, idx);                             \
    table->num_elems--;                                                        \
    return true;                                                               \
  }                                                                            \
  void hash_table_##NAME##_clear(struct hash_table_##NAME *table) {            \
    memset(table->elems, 0,                                                    \
           sizeof(struct hash_table_##NAME##_elem) * table->cap);              \
    memset(table->deleted, 0, table->cap / 8);                                 \
    table->num_elems = 0;                                                      \
  }

#endif // __HASH_H_
