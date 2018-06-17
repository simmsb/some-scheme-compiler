#ifndef SOMESCHEME_QUEUE_H
#define SOMESCHEME_QUEUE_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define RUNTIME_ERROR(S)                                                       \
    do {                                                                       \
        fprintf(stderr, "Runtime Error (%s:%d): %s\n", __func__, __LINE__,     \
                (S));                                                          \
        exit(1);                                                               \
    } while (0)

#ifdef DEBUG
#define DEBUG_LOG(F, ...)                                                      \
    do {                                                                       \
        printf("DEBUG (%s:%d): ", __func__, __LINE__);                         \
        printf((F), __VA_ARGS__);                                              \
    } while (0)
#else
#define DEBUG_LOG(...)                                                         \
    do {                                                                       \
    } while (0)
#endif

#define DEFINE_QUEUE(TYPE, TNAME)                                              \
    struct queue_##TNAME {                                                     \
        size_t len;                                                            \
        size_t head, tail;                                                     \
        TYPE *data;                                                            \
    };                                                                         \
    struct queue_##TNAME queue_##TNAME##_new(size_t);                          \
    void queue_##TNAME##_enqueue(struct queue_##TNAME *, TYPE);                \
    TYPE queue_##TNAME##_dequeue(struct queue_##TNAME *);                      \
    size_t queue_##TNAME##_len(struct queue_##TNAME *);

#define MAKE_QUEUE(TYPE, TNAME)                                                \
    struct queue_##TNAME queue_##TNAME##_new(size_t initial) {                 \
                                                                               \
        /* ensure atleast enough space for one element */                      \
        if (initial < 1) {                                                     \
            initial = 1;                                                       \
        }                                                                      \
                                                                               \
        TYPE *data = malloc(sizeof(TYPE) * initial);                           \
        return (struct queue_##TNAME){initial, 0, 0, data};                    \
    }                                                                          \
                                                                               \
    void queue_##TNAME##_enqueue(struct queue_##TNAME *queue, TYPE elem) {     \
        queue->data[queue->head++] = elem;                                     \
                                                                               \
        queue->head %= queue->len;                                             \
                                                                               \
        /* if the head now points to the tail, we need to grow here */         \
        if (queue->head == queue->tail) {                                      \
            /* queue full, need to expand */                                   \
            size_t old_size = queue->len;                                      \
                                                                               \
            size_t new_size = 1 + queue->len + (queue->len >> 2);              \
            queue->data = realloc(queue->data, new_size * sizeof(TYPE));       \
            queue->len = new_size;                                             \
            DEBUG_LOG("growing queue from %ld to %ld\n", old_size, new_size);  \
                                                                               \
            size_t delta_size = new_size - old_size;                           \
                                                                               \
            /* move tail section to end of queue */                            \
            memmove(&queue->data[queue->tail],                                 \
                    &queue->data[queue->tail + delta_size],                    \
                    (old_size - queue->tail) * sizeof(TYPE));                  \
            queue->tail += delta_size;                                         \
        }                                                                      \
    }                                                                          \
                                                                               \
    TYPE queue_##TNAME##_dequeue(struct queue_##TNAME *queue) {                \
        if (queue->tail == queue->head) {                                      \
            RUNTIME_ERROR("Popping from empty queue");                         \
        }                                                                      \
                                                                               \
        TYPE result = queue->data[queue->tail++];                              \
        queue->tail %= queue->len;                                             \
        return result;                                                         \
    }                                                                          \
                                                                               \
    size_t queue_##TNAME##_len(struct queue_##TNAME *queue) {                  \
                                                                               \
        /* case 1, the head is further up the queue than the tail              \
         *                                                                     \
         * elems with parens around number are used                            \
         * | 0 | (1) | (2) | (3) | 4 | 5 | 6 |                                 \
         *       ^ tail            ^ head                                      \
         * len = head - tail = 4 - 1 = 3                                       \
         */                                                                    \
        if (queue->head > queue->tail) {                                       \
            return queue->head - queue->tail;                                  \
        }                                                                      \
                                                                               \
        /* case 2, the tail is further up the queue than the head              \
         *                                                                     \
         * | (0) | 1 | 2 | (3) | (4) | (5) | (6) |                             \
         *         ^ head   ^ tail                                             \
         * len = head + array_len - tail = 1 + 7 - 3 = 5                       \
         */                                                                    \
        return queue->head + queue->len - queue->tail;                         \
    }

#endif // SOMESCHEME_QUEUE_H
