#![no_std]
#![feature(c_variadic)]

extern crate alloc;

use libc_alloc::LibcAlloc;

#[global_allocator]
static ALLOCATOR: LibcAlloc = LibcAlloc;

mod libc_support;
pub mod platform;
// pub mod sdk;
// pub mod teavm;

#[derive(Default)]
pub struct Data {
    // pub teavm: Option<teavm::TeaVM>,
}
