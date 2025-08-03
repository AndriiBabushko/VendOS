#define SDL_MAIN_HANDLED
#include <SDL2/SDL.h>
#include "lvgl.h"
#include "lv_sdl_mouse.h"
#include "lv_sdl_mousewheel.h"
#include "lv_sdl_keyboard.h"

#include "screens/screen1.h"

int main(void) {
    lv_init();

    lv_display_t * disp = lv_sdl_window_create(SDL_HOR_RES, SDL_VER_RES);
    lv_indev_t * mouse = lv_sdl_mouse_create();
    lv_indev_t * wheel = lv_sdl_mousewheel_create();
    lv_indev_t * kb = lv_sdl_keyboard_create();

    lv_scr_load(screen1_create());

    Uint32 last = SDL_GetTicks();
    while (1) {
        SDL_Delay(5);
        Uint32 curr = SDL_GetTicks();
        lv_tick_inc(curr - last);
        last = curr;
        lv_timer_handler();
    }

    return 0;
}