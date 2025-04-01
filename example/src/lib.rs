unsafe extern "C" {
    fn vexDisplayRectFill(x1: i32, y1: i32, x2: i32, y2: i32);
}

#[unsafe(no_mangle)]
extern "C" fn add(a: i32, b: i32) -> i32 {
    // unsafe {
    //     vexDisplayRectFill(50, 50, 150, 150);
    // }
    a + b
}