#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define LIMIT 25000000u
static uint32_t le32(const unsigned char *p) {
    return (uint32_t)p[0] | (uint32_t)p[1] << 8 |
           (uint32_t)p[2] << 16 | (uint32_t)p[3] << 24;
}
static int write32(uint32_t value) {
    unsigned char b[4] = {value & 255, (value >> 8) & 255,
                          (value >> 16) & 255, (value >> 24) & 255};
    return fwrite(b, 1, 4, stdout) == 4;
}
static int fail(const char *message) {
    fprintf(stderr, "bb-crop: %s\n", message);
    return 1;
}
int main(int argc, char **argv) {
    if (argc == 2 && strcmp(argv[1], "--version") == 0) {
        puts("bb-crop-v2-1");
        return 0;
    }
    if (argc != 7) return fail("expected x y width height screen-width screen-height");
    uint32_t args[6];
    for (int i = 0; i < 6; i++) {
        char *end;
        errno = 0;
        unsigned long n = strtoul(argv[i+1], &end, 10);
        if (errno || *end || !*argv[i+1] || argv[i+1][0] == '-' || n > 8192)
            return fail("invalid crop argument");
        args[i] = (uint32_t)n;
    }
    uint32_t x = args[0], y = args[1], w = args[2], h = args[3];
    uint32_t sw = args[4], sh = args[5];
    if (!w || !h || !sw || !sh || x+w > sw || y+h > sh || (uint64_t)sw*sh*4 > LIMIT)
        return fail("crop outside screen or unsupported size");
    size_t expected = (size_t)sw*sh*4;
    unsigned char *input = malloc(expected+17);
    if (!input) return fail("allocation failed");
    size_t count = fread(input, 1, expected+17, stdin);
    if (ferror(stdin) || (count != expected+12 && count != expected+16)) {
        free(input);
        return fail("incomplete or unsupported screencap payload");
    }
    uint32_t format = le32(input+8);
    if (le32(input) != sw || le32(input+4) != sh || (format != 1 && format != 2)) {
        free(input);
        return fail("screen dimensions or pixel format changed");
    }
    size_t offset = count-expected;
    int ok = fwrite("BBV2", 1, 4, stdout) == 4;
    uint32_t header[7] = {sw, sh, x, y, w, h, format};
    for (int i = 0; i < 7 && ok; i++) ok = write32(header[i]);
    for (uint32_t row = y; row < y+h && ok; row++) {
        const unsigned char *start = input+offset+((size_t)row*sw+x)*4;
        ok = fwrite(start, 4, w, stdout) == w;
    }
    free(input);
    if (!ok || fflush(stdout) != 0) return fail("output failed");
    return 0;
}
