#define SDL_MAIN_HANDLED
#include "lv_sdl_keyboard.h"
#include "lv_sdl_mouse.h"
#include "lv_sdl_mousewheel.h"
#include "lvgl.h"
#include "screens/screen2.h"
#include "screens/screen3.h"
#include "screens/screen1.h"

#include <SDL2/SDL.h>

int main(void) {
  lv_init();

  lv_display_t *disp = lv_sdl_window_create(SDL_HOR_RES, SDL_VER_RES);
  lv_indev_t *mouse = lv_sdl_mouse_create();
  lv_indev_t *wheel = lv_sdl_mousewheel_create();
  lv_indev_t *kb = lv_sdl_keyboard_create();

  screen1_init();
  screen2_init();
  screen3_init();

  lv_scr_load(screen1_get());

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
