# nexcopy-unlock, Rust port

A Rust port of the C tool in the parent directory. Same command line interface, same output, same
exit codes. It exists mainly to show the FFI shape needed to drive a legacy 32-bit vendor DLL from
Rust without pulling in a crate for it.

## It must be built 32-bit

`uDiskDLL.dll` is a 32-bit binary. A 64-bit process cannot load it, and `LoadLibrary` fails with
error 193, `ERROR_BAD_EXE_FORMAT`, before any Nexcopy code runs. This is the single most likely
thing to trip you up, because the source compiles perfectly well for x64 and then fails at runtime.

`.cargo/config.toml` pins the target, so a plain build is already correct:

```
rustup target add i686-pc-windows-msvc
cargo build --release
```

The binary lands in `target\i686-pc-windows-msvc\release\nexcopy_unlock.exe`.

## The DLL is not included

Obtain `uDiskDLL.dll` from Nexcopy and place it beside the built executable, in
`target\i686-pc-windows-msvc\release\`. Without it the tool exits with code 3.

## Notes

No crate dependencies. The Windows entry points are declared directly in a
`#[link(name = "kernel32")] extern "system"` block, and the five Nexcopy exports are resolved with
`GetProcAddress` and transmuted to matching function pointer types.

`extern "system"` matters here. The vendor's typedefs are `__stdcall`, which is a real calling
convention on 32-bit x86 and is ignored on x64. `extern "system"` resolves to stdcall on the i686
target, which is what makes the calls line up. `extern "C"` would build and then corrupt the stack.

See the parent [README](../README.md) for usage, exit codes and observed device behaviour.
