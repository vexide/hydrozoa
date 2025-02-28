use alloc::string::String;
use core::{
    alloc::Layout,
    ffi::{c_int, VaList},
};

use hashbrown::HashMap;
use vexide::core::{
    print,
    sync::{LazyLock, Mutex},
};

// TODO
// rust-lld: error: undefined symbol: os_mutex_destroy
// >>> referenced by bh_vector.c:273 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/shared/utils\bh_vector.c:273)
// >>>               bh_vector.c.obj:(bh_vector_destroy) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced by bh_hashmap.c:284 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/shared/utils\bh_hashmap.c:284)
// >>>               bh_hashmap.c.obj:(bh_hash_map_destroy) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced by ems_kfc.c:174 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/shared/mem-alloc/ems\ems_kfc.c:174)
// >>>               ems_kfc.c.obj:(gc_destroy_with_pool) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced 2 more times
// 
// rust-lld: error: undefined symbol: os_thread_jit_write_protect_np
// >>> referenced by aot_loader.c:4398 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/aot\aot_loader.c:4398)
// >>>               aot_loader.c.obj:(aot_load_from_aot_file) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced by aot_loader.c:4404 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/aot\aot_loader.c:4404)
// >>>               aot_loader.c.obj:(aot_load_from_aot_file) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// 
// rust-lld: error: undefined symbol: bsearch
// >>> referenced by aot_runtime.c:2297 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/aot\aot_runtime.c:2297)
// >>>               aot_runtime.c.obj:(aot_lookup_function) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced by wasm_native.c:191 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/common\wasm_native.c:191)
// >>>               wasm_native.c.obj:(lookup_symbol) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced by wasm_native.c:1524 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/common\wasm_native.c:1524)
// >>>               wasm_native.c.obj:(wasm_native_lookup_quick_aot_entry) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// 
// rust-lld: error: undefined symbol: os_mutex_init
// >>> referenced by bh_hashmap.c:66 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/shared/utils\bh_hashmap.c:66)
// >>>               bh_hashmap.c.obj:(bh_hash_map_create) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced by ems_kfc.c:17 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/shared/mem-alloc/ems\ems_kfc.c:17)
// >>>               ems_kfc.c.obj:(gc_init_internal) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced by wasm_runtime_common.c:6300 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/common\wasm_runtime_common.c:6300)
// >>>               wasm_runtime_common.c.obj:(wasm_externref_map_init) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// 
// rust-lld: error: undefined symbol: os_mutex_lock
// >>> referenced by bh_hashmap.c:93 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/shared/utils\bh_hashmap.c:93)
// >>>               bh_hashmap.c.obj:(bh_hash_map_insert) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced by bh_hashmap.c:141 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/shared/utils\bh_hashmap.c:141)
// >>>               bh_hashmap.c.obj:(bh_hash_map_find) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced by bh_hashmap.c:262 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/shared/utils\bh_hashmap.c:262)
// >>>               bh_hashmap.c.obj:(bh_hash_map_destroy) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced 6 more times
// 
// rust-lld: error: undefined symbol: os_mutex_unlock
// >>> referenced by bh_hashmap.c:117 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/shared/utils\bh_hashmap.c:117)
// >>>               bh_hashmap.c.obj:(bh_hash_map_insert) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced by bh_hashmap.c:123 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/shared/utils\bh_hashmap.c:123)
// >>>               bh_hashmap.c.obj:(bh_hash_map_insert) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced by bh_hashmap.c:151 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/shared/utils\bh_hashmap.c:151)
// >>>               bh_hashmap.c.obj:(bh_hash_map_find) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced 13 more times
// 
// rust-lld: error: undefined symbol: os_thread_get_stack_boundary
// >>> referenced by wasm_exec_env.c:279 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/common\wasm_exec_env.c:279)
// >>>               wasm_exec_env.c.obj:(wasm_exec_env_set_thread_info) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// 
// rust-lld: error: undefined symbol: os_self_thread
// >>> referenced by wasm_exec_env.c:284 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/common\wasm_exec_env.c:284)
// >>>               wasm_exec_env.c.obj:(wasm_exec_env_set_thread_info) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced by bh_log.c:33 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/shared/utils\bh_log.c:33)
// >>>               bh_log.c.obj:(bh_log) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// 
// rust-lld: error: undefined symbol: strchr
// >>> referenced by libc_builtin_wrapper.c:586 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/libraries/libc-builtin\libc_builtin_wrapper.c:586)
// >>>               libc_builtin_wrapper.c.obj:(strchr_wrapper) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// 
// rust-lld: error: undefined symbol: qsort
// >>> referenced by wasm_native.c:283 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/common\wasm_native.c:283)
// >>>               wasm_native.c.obj:(register_natives) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced by wasm_native.c:1479 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/common\wasm_native.c:1479)
// >>>               wasm_native.c.obj:(quick_aot_entry_init) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced by aot_runtime.c:1492 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/aot\aot_runtime.c:1492)
// >>>               aot_runtime.c.obj:(create_export_funcs) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced 1 more times
// 
// rust-lld: error: undefined symbol: strncpy
// >>> referenced by libc_builtin_wrapper.c:623 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/libraries/libc-builtin\libc_builtin_wrapper.c:623)
// >>>               libc_builtin_wrapper.c.obj:(strcpy_wrapper) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced by libc_builtin_wrapper.c:641 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/libraries/libc-builtin\libc_builtin_wrapper.c:641)
// >>>               libc_builtin_wrapper.c.obj:(strncpy_wrapper) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// 
// rust-lld: error: undefined symbol: malloc
// >>> referenced by vexos_platform.c:23 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/shared/platform/vexos\vexos_platform.c:23)
// >>>               vexos_platform.c.obj:(os_malloc) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// 
// rust-lld: error: undefined symbol: strncmp
// >>> referenced by libc_builtin_wrapper.c:609 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/libraries/libc-builtin\libc_builtin_wrapper.c:609)
// >>>               libc_builtin_wrapper.c.obj:(strncmp_wrapper) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced by aot_loader.c:3843 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/aot\aot_loader.c:3843)
// >>>               aot_loader.c.obj:(load_relocation_section) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced by aot_loader.c:3846 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/aot\aot_loader.c:3846)
// >>>               aot_loader.c.obj:(load_relocation_section) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced 12 more times
// 
// rust-lld: error: undefined symbol: vexos_dcache_invalidate
// >>> referenced by vexos_platform.c:83 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/shared/platform/vexos\vexos_platform.c:83)
// >>>               vexos_platform.c.obj:(os_dcache_flush) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// 
// rust-lld: error: undefined symbol: vexos_icache_invalidate
// >>> referenced by vexos_platform.c:89 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/shared/platform/vexos\vexos_platform.c:89)
// >>>               vexos_platform.c.obj:(os_icache_flush) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// 
// rust-lld: error: undefined symbol: atoi
// >>> referenced by libc_builtin_wrapper.c:710 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/libraries/libc-builtin\libc_builtin_wrapper.c:710)
// >>>               libc_builtin_wrapper.c.obj:(atoi_wrapper) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// >>> referenced by aot_loader.c:3133 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/aot\aot_loader.c:3133)
// >>>               aot_loader.c.obj:(do_text_relocation) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// 
// rust-lld: error: undefined symbol: strtol
// >>> referenced by libc_builtin_wrapper.c:733 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/libraries/libc-builtin\libc_builtin_wrapper.c:733)
// >>>               libc_builtin_wrapper.c.obj:(strtol_wrapper) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// 
// rust-lld: error: undefined symbol: strtoul
// >>> referenced by libc_builtin_wrapper.c:750 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/libraries/libc-builtin\libc_builtin_wrapper.c:750)
// >>>               libc_builtin_wrapper.c.obj:(strtoul_wrapper) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// 
// rust-lld: error: undefined symbol: memchr
// >>> referenced by libc_builtin_wrapper.c:765 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/libraries/libc-builtin\libc_builtin_wrapper.c:765)
// >>>               libc_builtin_wrapper.c.obj:(memchr_wrapper) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// 
// rust-lld: error: undefined symbol: strncasecmp
// >>> referenced by libc_builtin_wrapper.c:776 (D:/packages/cargo/git/checkouts/wamr-rust-sdk-1a60293b7f8ce5e8/c08ad83/crates/wamr-sys/wasm-micro-runtime/core/iwasm/libraries/libc-builtin\libc_builtin_wrapper.c:776)
// >>>               libc_builtin_wrapper.c.obj:(strncasecmp_wrapper) in archive D:\Rust\hydrozoa\target\armv7a-vex-v5\debug\deps\libwamr_sys-2b3c9481067b1d05.rlib
// 

// these really get more unhinged the more you read

#[no_mangle]
unsafe extern "C" fn abort() {
    panic!("abort");
}

#[allow(non_upper_case_globals)]
const max_align_t: usize = 16;

static LAYOUTS: Mutex<Option<HashMap<usize, Layout>>> = Mutex::new(None);

#[no_mangle]
extern "C" fn calloc(nmemb: usize, size: usize) -> *mut u8 {
    let layout = Layout::from_size_align(size * nmemb, max_align_t).unwrap();
    if layout.size() == 0 {
        return core::ptr::null_mut();
    }

    let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) };
    if ptr.is_null() {
        return ptr;
    }

    let mut layouts = LAYOUTS.try_lock().unwrap();
    layouts.get_or_insert_default().insert(ptr as usize, layout);

    ptr
}

#[no_mangle]
extern "C" fn free(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }
    let mut layouts = LAYOUTS.try_lock().unwrap();
    let layout = layouts
        .get_or_insert_default()
        .remove(&(ptr as usize))
        .expect("double free detected");
    unsafe { alloc::alloc::dealloc(ptr, layout) };
}

#[no_mangle]
extern "C" fn realloc(ptr: *mut u8, size: usize) -> *mut u8 {
    if ptr.is_null() {
        return calloc(1, size);
    }

    let mut layouts = LAYOUTS.try_lock().unwrap();
    let layout = layouts
        .get_or_insert_default()
        .remove(&(ptr as usize))
        .expect("realloc on unknown pointer");
    let new_layout = Layout::from_size_align(size, layout.align()).unwrap();
    let new_ptr = unsafe { alloc::alloc::realloc(ptr, layout, new_layout.size()) };
    if new_ptr.is_null() {
        return new_ptr;
    }

    layouts.get_or_insert_default().insert(new_ptr as usize, new_layout);

    new_ptr
}

#[no_mangle]
extern "C" fn strcmp(s1: *const u8, s2: *const u8) -> i32 {
    let mut i = 0;
    loop {
        let c1 = unsafe { *s1.add(i) };
        let c2 = unsafe { *s2.add(i) };
        if c1 == 0 && c2 == 0 {
            return 0;
        } else if c1 == 0 {
            return -1;
        } else if c2 == 0 {
            return 1;
        } else if c1 != c2 {
            return c1 as i32 - c2 as i32;
        }
        i += 1;
    }
}

#[no_mangle]
unsafe extern "C" fn printf(format: *const u8, mut args: ...) -> c_int {
    let mut s = String::new();
    let bytes_written = printf_compat::format(
        format,
        args.as_va_list(),
        printf_compat::output::fmt_write(&mut s),
    );
    print!("{}", s);
    bytes_written
}

#[no_mangle]
unsafe extern "C" fn snprintf(
    buffer: *mut u8,
    bufsz: usize,
    format: *const u8,
    mut args: ...
) -> c_int {
    vsnprintf(buffer, bufsz, format, args.as_va_list())
}

#[no_mangle]
unsafe extern "C" fn vsnprintf(
    buffer: *mut u8,
    bufsz: usize,
    format: *const u8,
    args: VaList,
) -> c_int {
    let mut s = String::new();
    let bytes_written =
        printf_compat::format(format, args, printf_compat::output::fmt_write(&mut s));

    s.truncate(bufsz - 1);
    s.push('\0');

    let bytes_written = bytes_written.min(bufsz as i32 - 1);
    core::ptr::copy_nonoverlapping(s.as_ptr(), buffer, bytes_written as usize);
    bytes_written
}

#[no_mangle]
extern "C" fn __popcountsi2(a: i32) -> i32 {
    let x = a as u32;
    let x = x - ((x >> 1) & 0x55555555);
    let x = ((x >> 2) & 0x33333333) + (x & 0x33333333);
    let x = (x + (x >> 4)) & 0x0F0F0F0F;
    let x = x + (x >> 16);
    ((x + (x >> 8)) & 0x0000003F) as i32
}

#[no_mangle]
extern "C" fn __popcountdi2(a: i64) -> i64 {
    let x = a as u64;
    let x = x - ((x >> 1) & 0x5555555555555555);
    let x = ((x >> 2) & 0x3333333333333333) + (x & 0x3333333333333333);
    let x = (x + (x >> 4)) & 0x0F0F0F0F0F0F0F0F;
    let x = x + (x >> 32);
    let x = x + (x >> 16);
    ((x + (x >> 8)) & 0x0000007F) as i64
}
