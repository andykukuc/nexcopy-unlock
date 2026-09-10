# nexcopy-unlock

A small Windows command line tool that queries and clears the write protection on a Nexcopy USB
drive by calling the vendor's `uDiskDLL.dll` directly, instead of going through the Nexcopy GUI.

Useful when you want the unlock step to be scriptable, to run unattended, or to return a real exit
code that a batch file can branch on.

## The DLL is not included

`uDiskDLL.dll` is Nexcopy's proprietary binary and is **not** redistributed here. Obtain it from
Nexcopy with your drive or duplicator software, and place it next to the compiled executable.
Without it the tool exits with code 3.

## Build

The DLL is 32-bit, so the executable **must** be built 32-bit. A 64-bit process cannot load it and
`LoadLibrary` fails with error 193, `ERROR_BAD_EXE_FORMAT`.

From an *x86 Native Tools Command Prompt for Visual Studio*:

```
cl /W4 /O2 nexcopy_unlock.c
```

## Usage

```
nexcopy_unlock.exe [--apply|--temporary|--lunx|--clear-sectors]
```

| Mode | DLL function called | Effect |
| --- | --- | --- |
| no argument | `GetWriteProtectStatusDLL` | Probe only. Reads status, changes nothing. |
| `--apply` | `DisableReadOnlyDLL` | Clears write protection permanently. |
| `--temporary` | `DisableReadOnlyTemporarilyDLL` | Clears write protection until the drive is re-plugged. |
| `--lunx` | `DisableWriteProtectLunXDLL` | Clears protection on the LUN-X device. |
| `--clear-sectors` | `SectorWriteProtectDLL` | Writes an empty protection-range table, clearing sector level protection. |

Exactly one mode may be given. Any other argument prints usage and exits 64.

Before touching the device the tool checks that the target volume carries the expected label and
refuses otherwise, so that a mistyped drive letter cannot send controller commands to the wrong
disk. After any change it re-probes and then performs a real write and delete test, so the output
tells you whether the drive is genuinely writable rather than whether the DLL claimed success.

## Exit codes

| Code | Meaning |
| --- | --- |
| 0 | Success |
| 2 | Volume label did not match, or the volume could not be read |
| 3 | `uDiskDLL.dll` could not be loaded |
| 4 | A required export was missing from the DLL |
| 5 | The status probe failed, so no change was attempted |
| 6 | The unlock call returned failure |
| 64 | Bad arguments |

## Observed behaviour

Recorded against one drive, kept in `results/`. Your hardware may differ.

- `GetWriteProtectStatusDLL` returned 1 with `status=-1` in every run, including after a successful
  unlock. The status value does not appear to reflect the actual protection state, so trust the
  write test rather than the reported status.
- `--clear-sectors` returned 0 with `detail=-2`, which looks like failure, yet the drive was
  writable immediately afterwards. This was the only mode that actually worked.
- `--temporary` reported success from the DLL, but the following write test failed with Win32 error
  19, `ERROR_WRITE_PROTECT`. The drive was still locked.
- `--lunx` crashed the process with an access violation, exit code `-1073741819`. Avoid it unless
  you have a LUN-X device and know what it expects.

## Warnings

`--apply` is described by the vendor as permanent. Treat it as not reversible by this tool.

The drive letter and the expected volume label are compile time constants near the top of
`nexcopy_unlock.c`. Change them to match your device before building.

## Licence

Provided as is, with no warranty. Nexcopy is not affiliated with this project, and `uDiskDLL.dll`
remains the property of its owner.
