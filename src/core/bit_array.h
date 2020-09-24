#ifndef __BIT_ARRAY_H_
#define __BIT_ARRAY_H_

#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

size_t index_in_bitarray(size_t idx);

size_t bit_index_in_bitarray(size_t idx);

uint8_t *bit_array_new(size_t num_bits);

/**
 * Get a bit from a bit array.
 */
bool get_bit_in_bitarray(uint8_t *bitarray, size_t idx);

/**
 * Set a bit in a bit array, returns the previous value.
 */
bool set_bit_in_bitarray(uint8_t *bitarray, size_t idx, bool val);

#endif // __BIT_ARRAY_H_
