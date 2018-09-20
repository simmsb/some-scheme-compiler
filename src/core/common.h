#ifndef SOMESCHEME_COMMON_H
#define SOMESCHEME_COMMON_H

#define RUNTIME_ERROR(F, ...)                                                  \
    do {                                                                       \
        fprintf(stderr, "Runtime Error (%s:%d): ", __func__, __LINE__);        \
        fprintf(stderr, F "\n" __VA_OPT__(, ) __VA_ARGS__);                   \
        exit(1);                                                               \
    } while (0)

#ifdef DEBUG
#define DEBUG_LOG(F, ...)                                                      \
    do {                                                                       \
        printf("DEBUG (%s:%d): ", __func__, __LINE__);                         \
        printf(stderr, (F "\n")__VA_OPT__(, ) __VA_ARGS__);                    \
    } while (0)
#else
#define DEBUG_LOG(...)                                                         \
    do {                                                                       \
    } while (0)
#endif

#endif // SOMESCHEME_COMMON_H
