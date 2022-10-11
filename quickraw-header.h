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

typedef struct Image {
  struct RustVec data;
  unsigned int width;
  unsigned int height;
} Image;

typedef struct QuickrawResponse_Image {
  bool has_error;
  char *error_msg;
  struct Image content;
} QuickrawResponse_Image;

struct QuickrawResponse_BasicInfo quickraw_load_basicinfo(char *cpath, bool with_thumbnail);

void quickraw_free_basicinfo(struct QuickrawResponse_BasicInfo response);

struct QuickrawResponse_Image quickraw_load_image(char *cpath);

void quickraw_free_image(struct QuickrawResponse_Image response);
