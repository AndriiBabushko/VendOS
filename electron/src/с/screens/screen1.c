#include "screen1.h"
#include "screen2.h"
#include "src/print/printer.h"

#include <misc/lv_timer_private.h>
#include <stdbool.h>
#include <stdio.h>

static lv_obj_t* scr1 = NULL;
static lv_obj_t* btn1 = NULL;
static lv_obj_t* btn_print = NULL;
static lv_obj_t* lbl_status = NULL;
static bool busy1 = false;
static lv_display_t * g_disp;

static void btn1_cb(lv_event_t* e);
static void print_btn_cb(lv_event_t* e);
static void unlock_timer_cb(lv_timer_t* t);

void screen1_init(void) {
    if (scr1) return;
    scr1 = lv_obj_create(NULL);

    lv_sdl_window_set_title(g_disp, "Screen 1");

    btn1 = lv_btn_create(scr1);
    lv_obj_align(btn1, LV_ALIGN_CENTER, 0, -40);
    lv_obj_t* label1 = lv_label_create(btn1);
    lv_label_set_text(label1, "Next Screen 2");
    lv_obj_add_event_cb(btn1, btn1_cb, LV_EVENT_SHORT_CLICKED, NULL);

    btn_print = lv_btn_create(scr1);
    lv_obj_align(btn_print, LV_ALIGN_CENTER, 0, 30);
    lv_obj_t* label2 = lv_label_create(btn_print);
    lv_label_set_text(label2, "Print photo");
    lv_obj_add_event_cb(btn_print, print_btn_cb, LV_EVENT_SHORT_CLICKED, NULL);

    lbl_status = lv_label_create(scr1);
    lv_obj_align(lbl_status, LV_ALIGN_BOTTOM_MID, 0, -8);
    lv_label_set_text(lbl_status, "");
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
    lv_sdl_window_set_title(g_disp, "Screen 2");

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

static void print_btn_cb(lv_event_t* e) {
    (void)e;

    const PrintOptions print_options = {
        .queue   = NULL,
        .title   = "VendOS example",
        .media   = NULL,
        .scaling = "fit",
        .copies  = 1
    };
    char err[256] = {0};
    const int job = print_png_via_cups("example.png", &print_options, err, sizeof err);

    char msg[160];
    if (job <= 0) {
        snprintf(msg, sizeof msg, "Print error: %s", err[0] ? err : "unknown");
    } else {
        snprintf(msg, sizeof msg, "Sent to printer, job #%d", job);
    }
    lv_label_set_text(lbl_status, msg);
}
