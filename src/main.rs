#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lzh_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

//on tablet use cargo run -- -display none -serial stdio
//need to use tablet for me to code outside my dorm lol

use core::panic::PanicInfo;
use x86_64::instructions::port::Port;
//use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use lzh_os::println;
use lzh_os::serial_println;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("Hello World1!");

    //handle interrupts
    lzh_os::init();


    //testing
    #[cfg(test)]
    test_main();
    
    println!("not crash!!!!!!!! fuck!!!");
    lzh_os::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use lzh_os::serial;
    println!("{}", info);
    serial_println!("{}",info);//this for debug in my tablet lol
    lzh_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    lzh_os::test_panic_handler(info)
}