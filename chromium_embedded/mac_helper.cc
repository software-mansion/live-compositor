#include <cstdlib>

#include <include/cef_app.h>
#include <include/wrapper/cef_library_loader.h>

int main(int argc, char* argv[]) {
    CefScopedLibraryLoader lib_loader;
    if (!lib_loader.LoadInHelper()) {
        return 1;  
    }

    CefMainArgs main_args(argc, argv);
  
    return CefExecuteProcess(main_args, nullptr, nullptr);
}
