#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdio.h>
#include <string.h>

typedef int (__stdcall *GetWriteProtectStatusFn)(char drive_letter, int *status);
typedef int (__stdcall *DisableReadOnlyFn)(char drive_letter);
typedef int (__stdcall *SectorWriteProtectFn)(char drive_letter,
                                               const unsigned __int64 *ranges,
                                               int range_count,
                                               int *detail_status);

static int test_write(char drive_letter) {
    char path[] = "X:\\__nexcopy_write_test.tmp";
    const char payload[] = "Nexcopy write test\r\n";
    DWORD written = 0;
    path[0] = drive_letter;
    DeleteFileA(path);
    HANDLE file = CreateFileA(path, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS,
                              FILE_ATTRIBUTE_NORMAL, NULL);
    if (file == INVALID_HANDLE_VALUE) {
        printf("Write test: FAILED opening file (Win32 error %lu)\n", GetLastError());
        return 0;
    }
    int ok = WriteFile(file, payload, (DWORD)(sizeof(payload) - 1), &written, NULL) &&
             written == sizeof(payload) - 1;
    DWORD error = ok ? ERROR_SUCCESS : GetLastError();
    CloseHandle(file);
    if (!ok) {
        printf("Write test: FAILED writing file (Win32 error %lu)\n", error);
        DeleteFileA(path);
        return 0;
    }
    printf("Write test: SUCCEEDED (%lu bytes)\n", written);
    printf("Delete test: %s\n", DeleteFileA(path) ? "SUCCEEDED" : "FAILED");
    return 1;
}

static int is_expected_volume(char drive_letter) {
    char root[] = "X:\\";
    char volume_name[MAX_PATH] = {0};
    DWORD serial = 0, max_component = 0, flags = 0;
    char fs_name[MAX_PATH] = {0};

    root[0] = drive_letter;
    if (!GetVolumeInformationA(root, volume_name, MAX_PATH, &serial,
                               &max_component, &flags, fs_name, MAX_PATH)) {
        fprintf(stderr, "GetVolumeInformation(%s) failed: %lu\n", root, GetLastError());
        return 0;
    }

    printf("Volume %s label=%s filesystem=%s\n", root, volume_name, fs_name);
    if (_stricmp(volume_name, "CVN7291B") != 0) {
        fprintf(stderr, "Refusing: expected volume label CVN7291B.\n");
        return 0;
    }
    return 1;
}

int main(int argc, char **argv) {
    const char drive_letter = 'I';
    int permanent = argc == 2 && strcmp(argv[1], "--apply") == 0;
    int temporary = argc == 2 && strcmp(argv[1], "--temporary") == 0;
    int lunx = argc == 2 && strcmp(argv[1], "--lunx") == 0;
    int clear_sectors = argc == 2 && strcmp(argv[1], "--clear-sectors") == 0;
    int apply = permanent || temporary || lunx || clear_sectors;
    int status = -1;

    if (argc > 2 || (argc == 2 && !apply)) {
        fprintf(stderr, "Usage: %s [--apply|--temporary|--lunx|--clear-sectors]\n", argv[0]);
        return 64;
    }
    if (!is_expected_volume(drive_letter)) return 2;

    HMODULE dll = LoadLibraryA("uDiskDLL.dll");
    if (!dll) {
        fprintf(stderr, "LoadLibrary(uDiskDLL.dll) failed: %lu\n", GetLastError());
        return 3;
    }

    GetWriteProtectStatusFn get_status =
        (GetWriteProtectStatusFn)GetProcAddress(dll, "GetWriteProtectStatusDLL");
    DisableReadOnlyFn disable_readonly =
        (DisableReadOnlyFn)GetProcAddress(dll, "DisableReadOnlyDLL");
    DisableReadOnlyFn disable_temporary =
        (DisableReadOnlyFn)GetProcAddress(dll, "DisableReadOnlyTemporarilyDLL");
    DisableReadOnlyFn disable_lunx =
        (DisableReadOnlyFn)GetProcAddress(dll, "DisableWriteProtectLunXDLL");
    SectorWriteProtectFn sector_write_protect =
        (SectorWriteProtectFn)GetProcAddress(dll, "SectorWriteProtectDLL");
    if (!get_status || !disable_readonly || !disable_temporary || !disable_lunx ||
        !sector_write_protect) {
        fprintf(stderr, "Required Nexcopy exports were not found.\n");
        FreeLibrary(dll);
        return 4;
    }

    int probe_result = get_status(drive_letter, &status);
    printf("GetWriteProtectStatusDLL returned %d; status=%d\n", probe_result, status);

    if (!apply) {
        puts("Probe only; no controller-setting change was requested.");
        FreeLibrary(dll);
        return probe_result ? 0 : 5;
    }
    if (!probe_result) {
        fprintf(stderr, "Refusing write call because the Nexcopy status probe failed.\n");
        FreeLibrary(dll);
        return 5;
    }

    int result;
    if (clear_sectors) {
        int detail = 0x7fffffff;
        puts("Calling SectorWriteProtectDLL with an empty protection-range table ...");
        result = sector_write_protect(drive_letter, NULL, 0, &detail);
        printf("SectorWriteProtectDLL returned %d; detail=%d\n", result, detail);
    } else if (lunx) {
        puts("Calling DisableWriteProtectLunXDLL for I: ...");
        result = disable_lunx(drive_letter);
        printf("DisableWriteProtectLunXDLL returned %d\n", result);
    } else if (temporary) {
        puts("Calling DisableReadOnlyTemporarilyDLL for I: ...");
        result = disable_temporary(drive_letter);
        printf("DisableReadOnlyTemporarilyDLL returned %d\n", result);
    } else {
        puts("Calling DisableReadOnlyDLL for I: ...");
        result = disable_readonly(drive_letter);
        printf("DisableReadOnlyDLL returned %d\n", result);
    }

    status = -1;
    probe_result = get_status(drive_letter, &status);
    printf("Post-call GetWriteProtectStatusDLL returned %d; status=%d\n",
           probe_result, status);
    test_write(drive_letter);
    FreeLibrary(dll);
    return result ? 0 : 6;
}
