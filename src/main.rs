#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lzh_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

//on tablet use cargo run -- -display none -serial stdio
//need to use tablet for me to code outside my dorm lol

use core::panic::PanicInfo;
use bootloader::bootinfo;
use bootloader::entry_point;
//use x86_64::instructions::port::Port;
//use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use lzh_os::println;
use lzh_os::serial_println;
use bootloader::BootInfo;

entry_point!(kernel_main);

fn kernel_main(boot_info : &'static BootInfo) -> ! {
    use x86_64::VirtAddr;
    use lzh_os::memory;
    use x86_64::structures::paging::Translate;

    println!("Hello World1!");

    //handle interrupts
    lzh_os::init();

    //accessing level 4 paging
    let phys_mem_offset = VirtAddr::new (boot_info.physical_memory_offset);
    let mapper = unsafe {memory::init(phys_mem_offset)};

    let addresses = [
        0xb8000,
        0x201008,
        0x0100_0020_1a10,
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys= mapper.translate_addr(virt);
        println!("{:?} -> {:?}",virt,phys);
    }



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