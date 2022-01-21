#include "wrapper.h"
#include "stdio.h"

#define LOGGER_BUFFER_SIZE 4096

static bool default_rusty_logger(log_level level, const char *message) {
    return false;
}

rusty_logger_t rusty_logger = &default_rusty_logger;

// Because rust does not support varargs on stable yet, we need to get
// around the requirement to pass a varargs function as the logger.
// What we do here is to use this C function as an adapter that is varargs, but it
// calls the rust logger using an already formatted string, meaning that we can define the
// "rusty" logger without varargs on the rust side.
bool logger_wrapper(unsigned int log_level, const char *fmt, ...) {
    char message[LOGGER_BUFFER_SIZE];
    va_list args;
    va_start(args, fmt);
    bool status = vsprintf_s(message, LOGGER_BUFFER_SIZE, fmt, args);
    va_end(args);

    status = status && rusty_logger(log_level, message);
    return status;
}

void hook_set_rusty_logger(rusty_logger_t logger) {
    if(logger == NULL) {
        rusty_logger = &default_rusty_logger;
        hook_set_logger_proc(NULL);
    } else {
        rusty_logger = logger;
        hook_set_logger_proc(logger_wrapper);
    }
}

