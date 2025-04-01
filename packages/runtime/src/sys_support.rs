use alloc::string::String;
use core::{
    alloc::Layout,
    ffi::{c_char, c_int, VaList},
};
use core::ffi::c_void;
use core::ptr::addr_of_mut;
use hashbrown::HashMap;
use vexide::{
    io::{print, Write},
    sync::{LazyLock, Mutex},
};

pub mod wamr;

#[link(name = "c")]
extern "C" {}
#[link(name = "nosys")]
extern "C" {}

#[no_mangle]
unsafe extern "C" fn hydrozoa_get_microseconds_since_boot() -> u64 {
    unsafe { vex_sdk::vexSystemPowerupTimeGet() }
}

#[no_mangle]
unsafe extern "C" fn hydrozoa_get_stack_boundary() -> *mut u8 {
    core::ptr::null_mut() // FIXME: get from linker script
}

static mut MEM_BREAK: *mut u8 = addr_of_mut!(__heap_start);

extern "C" {
    static mut __heap_start: u8;
    static mut __heap_end: u8;
    fn __errno() -> *mut c_int;
}

// Port of https://github.com/eblot/newlib/blob/master/libgloss/epiphany/sbrk.c
#[no_mangle]
unsafe extern "C" fn _sbrk(incr: i32) -> *mut c_void {
    unsafe {
        
        let new_break = MEM_BREAK;
        MEM_BREAK = MEM_BREAK.wrapping_offset(incr as isize);

        if MEM_BREAK < addr_of_mut!(__heap_end) {
            new_break as *mut c_void
        } else {
            // We have run out of memory
            const ENOMEM: c_int = 12;
            *__errno() = ENOMEM;
            (-1_isize) as *mut c_void
        }
    }
}

#[no_mangle]
unsafe extern "C" fn _write(file: c_int, ptr: *const c_char, len: c_int) -> c_int {
    let slice: &[u8] = unsafe {
        core::slice::from_raw_parts(ptr.cast(), len as usize)
    };

    if file == 1 || file == 2 {
        let mut out = vexide::io::stdout().try_lock().unwrap();
        return out.write(slice).unwrap() as c_int;
    }

    const ENOSYS: c_int = 88;
    unsafe {
        *__errno() = ENOSYS;
    }
    -1
}
