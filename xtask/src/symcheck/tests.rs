use super::*;

const CLEAN_DEFINED: &str = "00012340 T dxn_gpui_tizen_main\n";
const CLEAN_UNDEFINED: &str = "\
         w _ITM_deregisterTMCloneTable
         w _ITM_registerTMCloneTable
         U _Unwind_Backtrace@GCC_4.3.0
         U __aeabi_unwind_cpp_pr0@GCC_3.5
         w __clock_gettime64
         w __cxa_finalize@GLIBC_2.4
         U __errno_location@GLIBC_2.4
         w __gmon_start__
         U __tls_get_addr@GLIBC_2.3
         U dlopen@GLIBC_2.4
         U sqrtf@GLIBC_2.4
         U ecore_main_loop_thread_safe_call_async
         U ecore_wl2_window_new
         U ui_app_main
";

const CLEAN_NEEDED: &str = "\
File: libdxn_app.so
NeededLibraries [
  ld-linux.so.3
  libc.so.6
  libcapi-appfw-application.so.0
  libdl.so.2
  libecore.so.1
  libecore_wl2.so.1
  libgcc_s.so.1
  libm.so.6
  libpthread.so.0
]
";

fn run(defined: &str, undefined: &str) -> Vec<Violation> {
    check_library(defined, undefined, CLEAN_NEEDED, DEFAULT_ENTRY)
}

fn check_nm_output(defined: &str, undefined: &str, entry: &str) -> Vec<Violation> {
    check_library(defined, undefined, CLEAN_NEEDED, entry)
}

#[test]
fn clean_library_passes() {
    assert_eq!(run(CLEAN_DEFINED, CLEAN_UNDEFINED), []);
}

#[test]
fn second_export_fails_by_name() {
    let defined = format!("{CLEAN_DEFINED}00012400 T rust_helper\n00020000 D some_table\n");
    assert_eq!(
        run(&defined, CLEAN_UNDEFINED),
        [
            Violation::ExtraExport("rust_helper".into()),
            Violation::ExtraExport("some_table".into()),
        ]
    );
}

#[test]
fn weak_export_fails_by_name() {
    let defined = format!("{CLEAN_DEFINED}00012500 W __rust_probestack\n00030000 V weak_object\n");
    assert_eq!(
        run(&defined, CLEAN_UNDEFINED),
        [
            Violation::WeakExport("__rust_probestack".into()),
            Violation::WeakExport("weak_object".into()),
        ]
    );
}

#[test]
fn undocumented_import_fails_by_name() {
    let undefined = format!(
        "{CLEAN_UNDEFINED}         U tdm_client_create\n         U ecore_wl2_window_geometry_set\n         w __some_unknown_weak\n"
    );
    assert_eq!(
        run(CLEAN_DEFINED, &undefined),
        [
            Violation::UndocumentedImport("tdm_client_create".into()),
            Violation::UndocumentedImport("ecore_wl2_window_geometry_set".into()),
            Violation::UndocumentedImport("__some_unknown_weak".into()),
        ]
    );
}

#[test]
fn missing_entry_and_another_entry_name() {
    assert_eq!(
        run("", CLEAN_UNDEFINED),
        [Violation::MissingEntry(DEFAULT_ENTRY.into())]
    );
    let nsh = "0001 T nsh_tizen_main\n";
    assert_eq!(check_nm_output(nsh, CLEAN_UNDEFINED, "nsh_tizen_main"), []);
    assert_eq!(
        check_nm_output(nsh, CLEAN_UNDEFINED, DEFAULT_ENTRY),
        [
            Violation::MissingEntry(DEFAULT_ENTRY.into()),
            Violation::ExtraExport("nsh_tizen_main".into()),
        ]
    );
}

#[test]
fn local_symbols_and_versions_parse() {
    let parsed =
        parse_nm("00001 t local_fn\n         U memcpy@GLIBC_2.4\n00002 T exported@@VERS_1\n");
    assert_eq!(
        parsed[0],
        Symbol {
            name: "local_fn".into(),
            version: None,
            kind: 't'
        }
    );
    assert_eq!(parsed[1].version.as_deref(), Some("GLIBC_2.4"));
    assert_eq!(parsed[2].version.as_deref(), Some("VERS_1"));
    // `.dynsym` has no locals: whatever its letter, a defined dynamic symbol is an export.
    assert_eq!(
        run(
            &format!("{CLEAN_DEFINED}00001 t local_fn\n"),
            CLEAN_UNDEFINED
        ),
        [Violation::ExtraExport("local_fn".into())]
    );
    let message = Violation::UndocumentedImport("tdm_client_create".into()).to_string();
    assert!(message.contains("tdm_client_create") && message.contains("symbols.allow"));
}

#[test]
fn ifunc_and_unique_exports_count_as_exports() {
    let defined = format!("{CLEAN_DEFINED}00012600 i my_ifunc\n00030010 u unique_object\n");
    assert_eq!(
        run(&defined, CLEAN_UNDEFINED),
        [
            Violation::ExtraExport("my_ifunc".into()),
            Violation::ExtraExport("unique_object".into()),
        ]
    );
}

#[test]
fn needed_libraries_are_glibc_libgcc_loader_or_declared_tizen_ones() {
    assert_eq!(
        parse_needed(CLEAN_NEEDED),
        [
            "ld-linux.so.3",
            "libc.so.6",
            "libcapi-appfw-application.so.0",
            "libdl.so.2",
            "libecore.so.1",
            "libecore_wl2.so.1",
            "libgcc_s.so.1",
            "libm.so.6",
            "libpthread.so.0",
        ]
    );
    assert_eq!(run(CLEAN_DEFINED, CLEAN_UNDEFINED), []);
    let needed = CLEAN_NEEDED.replace(
        "  libm.so.6\n",
        "  libm.so.6\n  libresolv.so.2\n  libtbm.so.1\n  libecore_evas.so.1\n",
    );
    assert_eq!(
        check_library(CLEAN_DEFINED, CLEAN_UNDEFINED, &needed, DEFAULT_ENTRY),
        [
            Violation::UnexpectedLibrary("libresolv.so.2".into()),
            Violation::UnexpectedLibrary("libtbm.so.1".into()),
            Violation::UnexpectedLibrary("libecore_evas.so.1".into()),
        ]
    );
    let libs = tizen_libraries();
    assert!(
        libs.contains(&"ecore_wl2") && libs.contains(&"dlog"),
        "{libs:?}"
    );
    assert!(library_allowed("ld-linux-x86-64.so.2", &libs));
    assert!(library_allowed("ld-linux-aarch64.so.1", &libs));
    assert!(library_allowed("libdlog.so.0", &libs));
    assert!(library_allowed("libecore_imf.so", &libs));
    assert!(
        !library_allowed("libecore_imf_evas.so.1", &libs),
        "a prefix of a declared library is not enough"
    );
}
