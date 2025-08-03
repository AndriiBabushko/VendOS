#include "screen2.h"
#include "screen3.h"
#include <misc/lv_timer_private.h>
#include <stdbool.h>

static lv_obj_t* scr2 = NULL;
static lv_obj_t* btn2 = NULL;
static bool busy2 = false;

static void btn2_cb(lv_event_t* e);
static void unlock_timer_cb(lv_timer_t* t);

void screen2_init(void) {
    if (scr2)
        return;
    scr2 = lv_obj_create(NULL);

    btn2 = lv_btn_create(scr2);
    lv_obj_center(btn2);

    lv_obj_t* label = lv_label_create(btn2);
    lv_label_set_text(label, "Next Screen 3");

    lv_obj_add_event_cb(btn2, btn2_cb, LV_EVENT_SHORT_CLICKED, NULL);
}

lv_obj_t* screen2_get(void) { return scr2; }

static void btn2_cb(lv_event_t* e) {
    if (busy2)
        return;
    busy2 = true;

    lv_indev_t* indev = NULL;
    while ((indev = lv_indev_get_next(indev)) != NULL) {
        lv_indev_enable(indev, false);
    }

    lv_scr_load(screen3_get());

    const lv_timer_t* t = lv_timer_create(unlock_timer_cb, 100, &busy2);
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
