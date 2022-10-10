#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct RustVec {
  unsigned char *ptr;
  unsigned int len;
  unsigned int capacity;
} RustVec;

typedef struct BasicInfo {
  char *exif;
  struct RustVec thumbnail;
  unsigned char orientation;
} BasicInfo;

typedef struct QuickrawResponse_BasicInfo {
  bool has_error;
  char *error_msg;
  struct BasicInfo content;
} QuickrawResponse_BasicInfo;

struct QuickrawResponse_BasicInfo quickraw_load_basicinfo(char *cpath, bool with_thumbnail);

void quickraw_free_basicinfo(struct QuickrawResponse_BasicInfo response);
