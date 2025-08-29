#pragma once

#ifndef PRINTER_H
#define PRINTER_H

typedef struct PrintOptions {
    const char* queue;    // queue name; NULL -> default
    const char* title;    // job name; NULL -> "Job"
    const char* media;    // IPP media (ex. "na_index-4x6_4x6in"); NULL -> default.
    const char* scaling;  // "auto"|"fit"|"fill"|"none"; NULL -> "auto"
    int copies;           // >=1
} PrintOptions;

// Returns job-id (>0) or <0; err — reason (if it has)
int print_png_via_cups(const char* image_path, const PrintOptions* opts,
                       char* errbuf, int errbuf_sz);


#endif //PRINTER_H
