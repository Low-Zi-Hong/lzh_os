#![no_std]
#![no_main]

use core::panic::PanicInfo;
use lzh_os::{QemuExitCode, exit_qemu, serial_println, serial_print};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
// 1. 直接手动调用测试函数
    should_fail();

    // 2. 如果函数居然没挂（没 Panic），说明测试失败了
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

fn should_fail() {
    serial_print!("should_panic::should_fail...\t");
    // 3. 这里应该触发 Panic
    assert_eq!(0, 1);
}