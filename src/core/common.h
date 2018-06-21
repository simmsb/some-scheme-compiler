#ifndef SOMESCHEME_COMMON_H
#define SOMESCHEME_COMMON_H

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

#endif // SOMESCHEME_COMMON_H
