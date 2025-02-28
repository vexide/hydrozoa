#![no_main]
#![no_std]

use alloc::vec;
use alloc::vec::Vec;
use anyhow::Context;
use vexide::{core::program::exit, prelude::*};
use wamr_rust_sdk::function::Function;
use wamr_rust_sdk::instance::Instance;
use wamr_rust_sdk::module::Module;
use wamr_rust_sdk::runtime::Runtime;
use wamr_rust_sdk::value::WasmValue;
use runtime::platform;
use vexide_wasm_startup::{startup, CodeSignature, ProgramFlags, ProgramOwner, ProgramType};

extern crate alloc;

const STACK_SIZE: u32 = 1024 * 64;

fn main(_peripherals: Peripherals) {
    let runtime = Runtime::new().expect("Unable to create runtime");

    if let Err(mut err) = run(&runtime) {
        // if let Some(info) = store.take_error_info() {
        //     err = err.context(info);
        // }
        println!("\nError: {:?}", err);
    }
}

fn run(runtime: &Runtime) -> anyhow::Result<()> {
    let wasm_bytes = platform::read_user_program();

    // TODO: This clone effectively doubles the program space in memory.
    // See if there's a way to reduce this usage.
    let module = Module::from_vec(&runtime, Vec::from(wasm_bytes), "hydrozoa_module.wasm")
        .context("Unable to load module")?;

    let mut instance = Instance::new(&runtime, &module, STACK_SIZE).context("Unable to instantiate module")?;

    // teavm::link_teavm(&mut *store, &mut instance).context("Unable to link teavm")?;
    // sdk::link(&mut *store, &mut instance).context("Unable to link sdk")?;
    // 
    // teavm::teamvm_main(&mut *store, &mut instance, &[]).context("Unable to run main")?;

    let function = Function::find_export_func(&instance, "add")?;

    let params: Vec<WasmValue> = vec![WasmValue::I32(3), WasmValue::I32(6)];
    let result = function.call(&instance, &params)?;
    assert_eq!(result[0], WasmValue::I32(9));

    Ok(())
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
    exit();
}
