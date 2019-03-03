#ifndef SOMESCHEME_COMMON_H
#define SOMESCHEME_COMMON_H

#define RUNTIME_ERROR(F, ...)                                                  \
  do {                                                                         \
    fprintf(stderr, "Runtime Error (%s:%d): ", __func__, __LINE__);            \
    fprintf(stderr, F "\n", ##__VA_ARGS__);                                    \
    exit(1);                                                                   \
  } while (0)

#ifdef DEBUG
#define DEBUG_LOG(F, ...)                                                      \
  do {                                                                         \
    fprintf(stderr, "DEBUG (%s:%d): ", __func__, __LINE__);                    \
    fprintf(stderr, (F "\n"), ##__VA_ARGS__);                                  \
  } while (0)
#else
#define DEBUG_LOG(...)                                                         \
  do {                                                                         \
  } while (0)
#endif

#endif // SOMESCHEME_COMMON_H

#define ALLOC_SPRINTF(S, ...)                                                  \
  do {                                                                         \
    size_t needed = snprintf(NULL, 0, __VA_ARGS__) + 1;                        \
    char *buf = malloc(needed);                                                \
    sprintf(buf, __VA_ARGS__);                                                 \
    (S) = buf;                                                                 \
  } while (0)

#ifdef DEBUG
#define DEBUG_FPRINTF(...) fprintf(__VA_ARGS__)
#else
#define DEBUG_FPRINTF(...) (void)0
#endif // DEBUG

#ifdef DEBUG
#define DEBUG_ONLY(expr) (e)
#else
#define DEBUG_ONLY(expr) (0)
#endif // DEBUG
