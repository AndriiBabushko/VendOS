#include "screen3.h"
#include "screen1.h"

static void btn_event_cb(lv_event_t * e) {
  lv_scr_load(screen1_create());
}

lv_obj_t * screen3_create(void) {
  lv_obj_t * scr = lv_obj_create(NULL);
  lv_obj_set_size(scr, lv_pct(100), lv_pct(100));

  lv_obj_t * btn = lv_btn_create(scr);
  lv_obj_center(btn);
  lv_obj_add_event_cb(btn, btn_event_cb, LV_EVENT_CLICKED, NULL);

  lv_obj_t * label = lv_label_create(btn);
  lv_label_set_text(label, "Go to Screen 1");
  lv_obj_center(label);

  return scr;
}