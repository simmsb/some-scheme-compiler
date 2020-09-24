#include "bit_array.h"

#include <stdio.h>

size_t index_in_bitarray(size_t idx) { return idx / 8; }

size_t bit_index_in_bitarray(size_t idx) { return idx % 8; }

uint8_t *bit_array_new(size_t num_bits) {
  size_t num_bytes = (num_bits + 8) / 8;

  return calloc(num_bytes, sizeof(uint8_t));
}

bool get_bit_in_bitarray(uint8_t *bitarray, size_t idx) {
  size_t arr_idx = index_in_bitarray(idx);
  size_t bit_idx = bit_index_in_bitarray(idx);

  return bitarray[arr_idx] & (1 << bit_idx);
}

bool set_bit_in_bitarray(uint8_t *bitarray, size_t idx, bool val) {
  size_t arr_idx = index_in_bitarray(idx);
  size_t bit_idx = bit_index_in_bitarray(idx);

  bool old_val = get_bit_in_bitarray(bitarray, idx);

  bitarray[arr_idx] = (bitarray[arr_idx] & ~(1 << bit_idx)) | (val << bit_idx);

  return old_val;
}
