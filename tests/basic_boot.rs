#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(lzh_os::test_runner)]

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    lzh_os::test_panic_handler(info)
}

use core::panic::PanicInfo;

#[unsafe(no_mangle)] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

fn test_runner(tests: &[&dyn Fn()]) {
    unimplemented!();
}

use lzh_os::println;

#[test_case]
fn test_println() {
    println!("test_println output");
}