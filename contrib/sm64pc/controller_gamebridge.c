#include <ctype.h>
#include <string.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <ultra64.h>
#include <assert.h>
#include <sys/types.h>
#include <sys/stat.h>

#include "controller_api.h"
#include "../configfile.h"

static FILE *vblank;
static FILE *input;

#define vblank_fname "vblank"
#define input_fname "input"
#define ok "OK\n"
#define bye "BYE"

static void gamebridge_close(void) {
    if (!configGameBridge) {
        return;
    }

    printf("\n[gamebridge] exiting\n");
    fwrite(bye, 1, strlen(bye), vblank);
    fclose(vblank);
    fclose(input);

    unlink(vblank_fname);
    unlink(input_fname);
}

static void gamebridge_init(void) {
    if (!configGameBridge) {
        return;
    }

    printf("[gamebridge] starting...\n");
    fflush(stdout);

    unlink(vblank_fname);
    unlink(input_fname);

    int result;

    result = mkfifo(vblank_fname, S_IRUSR|S_IWUSR);
    if (result < 0) {
        perror("mkfifo "vblank_fname);
        assert(result < 0);
    }

    result = mkfifo(input_fname, S_IRUSR| S_IWUSR);
    if (result < 0) {
       perror("mkfifo "input_fname);
       assert(result < 0);
    }

    vblank = fopen(vblank_fname, "w+");
    input = fopen(input_fname, "rb+");
    assert(vblank);
    assert(input);

    setvbuf(vblank, NULL, _IONBF, 0);
    setvbuf(input, NULL, _IONBF, 0);

    printf("[gamebridge] starting rust daemon\n");
    fflush(stdout);
    system("gamebridge &");
    atexit(gamebridge_close);
}

static void gamebridge_read(OSContPad *pad) {
    if (!configGameBridge) {
        return;
    }

    //printf("[gamebridge] waiting for input\n");
    fwrite(ok, 1, strlen(ok), vblank);
    uint8_t bytes[4] = {0};
    fread(bytes, 1, 4, input);
    pad->button = (bytes[0] << 8) | bytes[1];
    pad->stick_x = bytes[2];
    pad->stick_y = bytes[3];
    //printf("[gamebridge] %02x%02x %02x%02x\n", bytes[0], bytes[1], bytes[2], bytes[3]);
    fflush(stdout);
}

struct ControllerAPI controller_gamebridge = {
    gamebridge_init,
    gamebridge_read
};

