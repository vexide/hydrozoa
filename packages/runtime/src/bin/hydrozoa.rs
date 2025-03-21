#![no_main]
#![no_std]

use alloc::vec;
use alloc::vec::Vec;
use anyhow::Context;
use vexide::{program::exit, prelude::*};
use wamr_rust_sdk::function::Function;
use wamr_rust_sdk::instance::Instance;
use wamr_rust_sdk::module::Module;
use wamr_rust_sdk::runtime::Runtime;
use wamr_rust_sdk::value::WasmValue;
use runtime::{platform, sdk};
use vexide_wasm_startup::{startup, CodeSignature, ProgramFlags, ProgramOwner, ProgramType};

extern crate alloc;

const STACK_SIZE: u32 = 1024 * 64;

fn main(_peripherals: Peripherals) {
    run();
}

fn run() {
    let wasm_bytes = platform::read_user_program();
    
    let mut runtime = Runtime::builder();
    runtime = sdk::link(runtime);
    let runtime = runtime.build().unwrap();

    let module = Module::from_mut_slice(&runtime, wasm_bytes, "hydrozoa_module.wasm")
        .expect("Unable to load module");

    let mut instance = Instance::new(&runtime, &module, STACK_SIZE).expect("Unable to instantiate module");

    // teavm::link_teavm(&mut *store, &mut instance).context("Unable to link teavm")?;
    // sdk::link(&mut *store, &mut instance).context("Unable to link sdk")?;
    // 
    // teavm::teamvm_main(&mut *store, &mut instance, &[]).context("Unable to run main")?;

    let function = Function::find_export_func(&instance, "add").unwrap();

    let params: Vec<WasmValue> = vec![WasmValue::I32(3), WasmValue::I32(6)];
    let result = function.call(&mut instance, &params).unwrap();
    println!("add(3, 6) = {result:?}");
    // assert_eq!(result[0], WasmValue::I32(9));
}

#[link_section = ".code_signature"]
#[used]
static CODE_SIGNATURE: CodeSignature = CodeSignature::new(
    ProgramType::User,
    ProgramOwner::Partner,
    ProgramFlags::empty(),
);

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    startup();
    main(Peripherals::take().unwrap());
    // exit();
    loop {
        unsafe {
            vex_sdk::vexTasksRun();
        }
    }
}
