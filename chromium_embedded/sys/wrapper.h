#include "include/capi/cef_app_capi.h"
#include "include/capi/cef_base_capi.h"
#include <include/capi/cef_browser_capi.h>

#ifdef _WIN32
#include <include/internal/cef_types_win.h>
#endif // _WIN32

#ifdef __APPLE__
#include <include/wrapper/cef_library_loader.h>
#include <include/internal/cef_types_mac.h>
#endif // __APPLE__
