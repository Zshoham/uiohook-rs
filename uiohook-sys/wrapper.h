#include "libuiohook/include/uiohook.h"

typedef bool (*rusty_logger_t)(log_level, const char *);

extern rusty_logger_t rusty_logger;

void hook_set_rusty_logger(rusty_logger_t logger);