#include "screen3.h"
#include "screen1.h"
#include <misc/lv_timer_private.h>
#include <stdbool.h>

static lv_obj_t* scr3 = NULL;
static lv_obj_t* btn3 = NULL;
static bool busy3 = false;

static void btn3_cb(lv_event_t* e);
static void unlock_timer_cb(lv_timer_t* t);

void screen3_init(void) {
    if (scr3)
        return;
    scr3 = lv_obj_create(NULL);

    btn3 = lv_btn_create(scr3);
    lv_obj_center(btn3);

    lv_obj_t* label = lv_label_create(btn3);
    lv_label_set_text(label, "Next Screen 1");

    lv_obj_add_event_cb(btn3, btn3_cb, LV_EVENT_SHORT_CLICKED, NULL);
}

lv_obj_t* screen3_get(void) { return scr3; }

static void btn3_cb(lv_event_t* e) {
    if (busy3)
        return;
    busy3 = true;

    lv_indev_t* indev = NULL;
    while ((indev = lv_indev_get_next(indev)) != NULL) {
        lv_indev_enable(indev, false);
    }

    lv_scr_load(screen1_get());

    const lv_timer_t* t = lv_timer_create(unlock_timer_cb, 100, &busy3);
    (void)t;
}

static void unlock_timer_cb(lv_timer_t* t) {
    bool* busy = (bool*)t->user_data;
    *busy = false;

    lv_indev_t* indev = NULL;
    while ((indev = lv_indev_get_next(indev)) != NULL) {
        lv_indev_enable(indev, true);
    }

    lv_timer_del(t);
}
