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
use lzh_os::memory::BootInfoFrameAllocator;
use lzh_os::println;
use lzh_os::serial_println;
use bootloader::BootInfo;
use alloc::{boxed::Box,vec,vec::Vec,rc::Rc};

extern crate alloc;

entry_point!(kernel_main);

fn kernel_main(boot_info : &'static BootInfo) -> ! {
    use lzh_os::allocator;
    use x86_64::VirtAddr;
    use lzh_os::memory;

    println!("Hello World!");

    //handle interrupts
    lzh_os::init();

    //accessing level 4 paging
    let phys_mem_offset = VirtAddr::new (boot_info.physical_memory_offset);
    let mut mapper = unsafe {memory::init(phys_mem_offset)};
    let mut frame_allocator = unsafe {BootInfoFrameAllocator::init(&boot_info.memory_map)};

    //init heap
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap init fail!");

    ////map unuse page
    //let page = Page::containing_address(VirtAddr::new(0));
    //memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    ////write string 'New!' to tge screen through new mapping!
    //let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    //unsafe {page_ptr.offset(400).write_volatile(0x_f921_f077_5265_804e);}
    ////0x4e = 'N' 0x65 e 0x77 w 0x21 ! 0xf0 白底黑字

    use bootinfo::MemoryRegionType;
        println!("Memory map");
        for r in boot_info.memory_map.iter() {
            if r.region_type == MemoryRegionType::Usable {
                println!("{:?}",r);
            }
        }

    let heap_value = Box::new(41);
    println!("heap value at {:p}",heap_value);

    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("Vec at {:p}",vec.as_slice());

    let reference_counted = Rc::new(vec![1,2,3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));

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