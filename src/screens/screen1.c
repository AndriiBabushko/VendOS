#include "screen1.h"
#include "screen2.h"
#include <misc/lv_timer_private.h>
#include <stdbool.h>

static lv_obj_t* scr1 = NULL;
static lv_obj_t* btn1 = NULL;
static bool busy1 = false;

static void btn1_cb(lv_event_t* e);
static void unlock_timer_cb(lv_timer_t* t);

void screen1_init(void) {
    if (scr1)
        return;
    scr1 = lv_obj_create(NULL);

    btn1 = lv_btn_create(scr1);
    lv_obj_center(btn1);

    lv_obj_t* label = lv_label_create(btn1);
    lv_label_set_text(label, "Next Screen 2");

    lv_obj_add_event_cb(btn1, btn1_cb, LV_EVENT_SHORT_CLICKED, NULL);
}

lv_obj_t* screen1_get(void) { return scr1; }

static void btn1_cb(lv_event_t* e) {
    if (busy1)
        return;
    busy1 = true;

    lv_indev_t* indev = NULL;
    while ((indev = lv_indev_get_next(indev)) != NULL) {
        lv_indev_enable(indev, false);
    }

    lv_scr_load(screen2_get());

    lv_timer_t* t = lv_timer_create(unlock_timer_cb, 100, &busy1);
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
