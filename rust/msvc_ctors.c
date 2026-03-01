// Bridge GCC .ctors to MSVC .CRT$XCU.
//
// Go's linker puts the Go runtime startup function (_rt0_amd64_windows_lib)
// into a .ctors section when building a c-archive. GCC's linker processes
// .ctors automatically, but MSVC's linker does not — it only processes
// .CRT$X* sections. This registers _rt0_amd64_windows_lib as a .CRT$XCU
// initializer so the Go runtime starts before main().
//
// See https://github.com/golang/go/issues/42190
// See https://learn.microsoft.com/en-us/cpp/c-runtime-library/crt-initialization

extern void _rt0_amd64_windows_lib(void);

#pragma section(".CRT$XCU", read)
__declspec(allocate(".CRT$XCU")) void (*__go_init)(void) = _rt0_amd64_windows_lib;
#pragma comment(linker, "/include:__go_init")
