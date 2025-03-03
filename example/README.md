# Compilation

Install wamrc from <https://github.com/bytecodealliance/wasm-micro-runtime/releases/tag/WAMR-2.2.0>.

```shell
cargo install --git https://github.com/vexide/hydrozoa-cli --locked
cargo build --release
wamrc -v=5 --target=armv7 --target-abi=eabihf --cpu=cortex-a9 --cpu-features=+v7,+neon,+vfp3d16,+thumb2 --mllvm=--target=armv7a-none-eabi -o example.aot target\wasm32-unknown-unknown\release\example.wasm
hydrozoa upload --slot 1 --runtime ../target/armv7a-vex-v5/release/hydrozoa.bin ./example.aot
```