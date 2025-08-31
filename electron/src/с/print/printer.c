#include "printer.h"
#include <cups/cups.h>
#include <stdarg.h>
#include <stdio.h>

static void set_err(char* buf, int n, const char* fmt, ...) {
    if (!buf || n <= 0) return;
    va_list ap; va_start(ap, fmt);
    vsnprintf(buf, n, fmt, ap);
    va_end(ap);
}

int print_png_via_cups(const char* image_path, const PrintOptions* in_opts,
                       char* err, int err_sz)
{
    if (!image_path) { set_err(err, err_sz, "image_path is null"); return -1; }

    PrintOptions def;
    def.queue   = NULL;
    def.title   = "VendOS Job";
    def.media   = NULL;
    def.scaling = "auto";
    def.copies  = 1;

    const PrintOptions* opts = in_opts ? in_opts : &def;

    // 1) choose printer: certain or default
    cups_dest_t* destinaitons = NULL;
    const int num_destinations = cupsGetDests2(CUPS_HTTP_DEFAULT, &destinaitons);
    if (num_destinations <= 0) { set_err(err, err_sz, "No CUPS destinations"); return -2; }

    const cups_dest_t* dest = NULL;
    if (opts->queue && *opts->queue) dest = cupsGetDest(opts->queue, NULL, num_destinations, destinaitons);
    else                             dest = cupsGetDest(NULL,       NULL, num_destinations, destinaitons);

    if (!dest) {
        cupsFreeDests(num_destinations, destinaitons);
        set_err(err, err_sz, "No default printer");
        return -3;
    }

    // 2) print options
    cups_option_t* options = NULL;
    int num_options = 0;

    if (opts->copies > 1) {
        char tmp[16]; snprintf(tmp, sizeof tmp, "%d", opts->copies);
        num_options = cupsAddOption("copies", tmp, num_options, &options);
    }
    if (opts->scaling && *opts->scaling) {
        num_options = cupsAddOption("print-scaling", opts->scaling, num_options, &options);
    }
    if (opts->media && *opts->media) {
        num_options = cupsAddOption("media", opts->media, num_options, &options);
    }

    num_options = cupsAddOption("fit-to-page", "true", num_options, &options);

    // 3) printing .png
    const char* title = (opts->title && *opts->title) ? opts->title : "VendOS Job";
    const int job = cupsPrintFile(dest->name, image_path, title, num_options, options);

    cupsFreeOptions(num_options, options);
    cupsFreeDests(num_destinations, destinaitons);

    if (job <= 0) {
        set_err(err, err_sz, "cupsPrintFile failed: %s", cupsLastErrorString());
        return -4;
    }
    return job;
}
