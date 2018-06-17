#include <stdlib.h>
#include <stdio.h>
#include <assert.h>

#include "queue.h"

DEFINE_QUEUE(int, int)
MAKE_QUEUE(int, int)

int main() {
    struct queue_int queue = queue_int_new(0);

    for (int i = 0; i < 100; i++) {
        queue_int_enqueue(&queue, i);
    }

    for (int i = 0; i < 100; i++) {
        assert(queue_int_len(&queue) > 0);
        int result = queue_int_dequeue(&queue);
        printf("dequeued %d expecting %d\n", result, i);
        assert(result == i);
    }

    puts("yeah we're good");
}
