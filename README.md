# LZH_OS

A small 64-bit operating system kernel written from scratch in Rust.

LZH_OS is a freestanding `#![no_std]` kernel for `x86_64`. It boots on bare
metal, takes over the VGA text console and the serial port, sets up its own
interrupt table, page tables, frame allocator and heap, and then runs Rust
code on the heap — `Box`, `Vec` and `Rc` all work. It is a learning project
built along the lines of the *Writing an OS in Rust* series, with extra
pieces of my own (swappable heap allocators, PS/2 keyboard input, a serial
console kept around for debugging from a tablet).

Current status: a working hobby kernel. It boots, prints, handles interrupts,
maps memory and allocates on the heap. There is no userspace, no scheduler
and no filesystem yet.

## Features

**Boot and runtime**
- Freestanding `#![no_std]` / `#![no_main]` kernel with a hand-written target
  specification (`.cargo/x86_64-lzh_os.json`): `os = "none"`, soft-float,
  MMX/SSE disabled, red zone disabled, `rust-lld` as the linker.
- Booted by the `bootloader` crate (0.9.x) through the `bootimage` tool, which
  produces a raw BIOS-bootable disk image.
- `panic = "abort"`, with a panic handler that dumps to both the screen and
  the serial port before halting the CPU in a `hlt` loop.
- Kernel image base at `0x0020_0000` (2 MiB).

**Text output**
- VGA text mode console: an 80x25 character buffer with scrolling, 16-colour
  foreground/background support, wrapped in a spin lock and accessed through
  volatile writes.
- `print!` / `println!` macros, usable anywhere in the kernel.
- Serial console on COM1 (`0x3F8`) with `serial_print!` / `serial_println!`,
  used by the test framework and handy for debugging on machines without a
  usable display (see *Running on a tablet / headless* below).

**Interrupts and CPU setup**
- Global Descriptor Table with a kernel code segment and a TSS, plus a
  dedicated 20 KiB double-fault IST stack.
- Interrupt Descriptor Table with handlers for breakpoints, double faults
  (routed to the IST so stack overflow does not triple-fault), and page
  faults — the page fault handler reports the accessed address from `CR2`,
  the error code and the full stack frame.
- 8259 PIC remapped to interrupt vectors 32 and 40, with end-of-interrupt
  acknowledged after every hardware IRQ.
- Timer interrupt (prints a heartbeat) and PS/2 keyboard input decoded with
  `pc-keyboard` (US 104-key layout, scan code set 1).

**Memory management**
- Access to the active level-4 page table through the bootloader's physical
  memory offset, wrapped in the `x86_64` crate's `OffsetPageTable`.
- Virtual-to-physical address translation by walking all four page table
  levels, and a small example mapping (`create_example_mapping`) that maps a
  page to the VGA frame buffer.
- `BootInfoFrameAllocator`, which hands out free 4 KiB frames straight from
  the memory map supplied by the bootloader. The kernel prints the usable
  memory regions it finds at boot.

**Heap allocation**
- A 100 KiB heap mapped at `0x4444_4444_0000` during boot, with a global
  allocator installed so the `alloc` crate works: the kernel allocates a
  `Box`, pushes 500 integers into a `Vec`, and clones an `Rc`.
- Three allocator implementations to choose from at compile time (see below).

**Testing**
- A custom test framework that runs *inside* the kernel, with results
  reported back through QEMU's debug exit device.
- Integration tests for heap allocation, deliberate stack overflow (to prove
  the double-fault handler works) and panic handling, plus in-kernel unit
  tests for console output and breakpoint exceptions.

## Choosing a heap allocator

Three allocators are implemented. Exactly one is compiled as the global
allocator, selected with a Cargo feature:

| Feature | Allocator | Notes |
| --- | --- | --- |
| `_BUMP_` | `BumpAllocator` | Simple pointer bump. Frees are only reclaimed once every outstanding allocation is gone. |
| `_LINKEDLIST_` | `LinkedListAllocator` | Free-region list with first-fit search, splitting on allocate and merging on free. Experimental and not yet used by default. |
| `_FIXEDSIZEBLOCK_` | `FixedSizeBlockAllocator` | **Default.** Nine block-size classes (8 B to 2 KiB) served from free lists, with `linked_list_allocator` as the fallback for larger or highly aligned allocations. |

Build with a different allocator by disabling the default feature set:

```sh
cargo run --no-default-features --features _BUMP_
cargo run --no-default-features --features _LINKEDLIST_
cargo run --no-default-features --features _FIXEDSIZEBLOCK_
```

At boot the kernel prints which allocator is in use.

## Project structure

```
lzh_os/
├── Cargo.toml                     crate metadata, allocator features, bootimage test config
├── Cargo.lock                     pinned dependency versions
├── .cargo/
│   ├── config.toml                build target, bootimage runner, nightly build-std settings
│   └── x86_64-lzh_os.json         custom bare-metal target specification
├── src/
│   ├── main.rs                    kernel entry point: boot info, paging, heap, demo allocations
│   ├── lib.rs                     shared library root: init(), test runner, QEMU exit codes, hlt_loop
│   ├── vga_buffer.rs              VGA text mode console: colours, scrolling, print!/println!
│   ├── serial.rs                  COM1 serial port: serial_print!/serial_println!
│   ├── gdt.rs                     GDT and TSS, double-fault IST stack
│   ├── interrupts.rs              IDT, exception handlers, PIC setup, timer and keyboard IRQs
│   ├── memory.rs                  page table access and translation, frame allocator
│   ├── allocator.rs               heap initialisation, Locked wrapper, global allocator selection
│   ├── allocator/
│   │   ├── bump.rs                bump allocator
│   │   ├── linked_list.rs         linked-list allocator
│   │   └── fixed_size_block.rs    fixed-size block allocator (default)
│   └── vga_buffer.txt             early draft of the console code, kept for reference
└── tests/
    ├── basic_boot.rs              early-series boot scaffolding
    ├── heap_allocation.rs         Box/Vec allocation and long-lived-box tests
    ├── should_panic.rs            verifies the test harness detects panics
    └── stack_overflow.rs          triggers a stack overflow, checks the double-fault handler
```

## Requirements

- A recent **nightly** Rust toolchain. The kernel uses unstable features
  (`custom_test_frameworks`, `abi_x86_interrupt`, `build-std`,
  `panic-abort-tests`).
- The `rust-src` and `llvm-tools-preview` components (`build-std` and the
  bundled linker need them).
- [`bootimage`](https://crates.io/crates/bootimage) for building the bootable
  disk image.
- QEMU (`qemu-system-x86_64`) to run and test the kernel without rebooting
  your machine.

On macOS or Linux with `rustup`:

```sh
rustup toolchain install nightly
rustup component add rust-src llvm-tools-preview
cargo install bootimage
```

Install QEMU through your package manager, for example `brew install qemu`
(macOS) or `sudo apt install qemu-system-x86` (Debian/Ubuntu).

There is no `rust-toolchain.toml` pinning a specific nightly, so if the build
fails on an unstable feature, update the toolchain with
`rustup update nightly`.

## Build and run

```sh
cargo build                 # build the kernel
cargo run                   # build and boot it in QEMU
cargo bootimage             # build the bootable disk image only
cargo build --release       # optimised build
```

`cargo run` works because `.cargo/config.toml` registers `bootimage runner` as
the runner for the bare-metal target.

### Running on a tablet / headless

To run without a graphical window, with everything printed to the terminal
over the serial port:

```sh
cargo run -- -display none -serial stdio
```

Extra arguments after `--` are passed to QEMU.

### Booting on real hardware from a USB pendrive

> **Legacy BIOS only.** The image produced by `bootimage` is a BIOS/MBR disk
> image. It does **not** support UEFI, and it will not boot on machines that
> have dropped legacy/CSM boot support.

1. Build the image:

   ```sh
   cargo bootimage
   ```

   The raw disk image is written to:

   ```
   target/x86_64-lzh_os/debug/bootimage-lzh_os.bin
   ```

   (with `cargo bootimage --release`, look in `target/x86_64-lzh_os/release/`).

2. Write it to the pendrive. **Pick the device carefully — these commands
   destroy whatever is on the target device.** Find the USB device first, and
   make sure it is not your system disk.

   macOS — check with `diskutil list`, then (for `disk4`, choose your own):

   ```sh
   diskutil unmountDisk /dev/disk4
   sudo dd if=target/x86_64-lzh_os/debug/bootimage-lzh_os.bin of=/dev/rdisk4 bs=1m
   diskutil eject /dev/disk4
   ```

   Linux — check with `lsblk`, then (for `sdb`, choose your own):

   ```sh
   sudo dd if=target/x86_64-lzh_os/debug/bootimage-lzh_os.bin of=/dev/sdb bs=4M status=progress conv=fsync
   sync
   ```

   Windows — write the image with Rufus or balenaEtcher (in raw/DD mode if
   asked, and rename the file to `.img` if the tool refuses `.bin`).

3. Boot the machine from the pendrive:

   - Enter the firmware setup (usually `Del`, `F2` or `F12`) and select
     **Legacy BIOS** / **CSM** boot — the kernel will not appear in the UEFI
     boot menu.
   - Disable Secure Boot if it blocks the drive.
   - Make sure the USB device is first in the boot order, then save and
     restart.

   The screen starts with `Hello World!`, the allocator in use, and the usable
   memory regions found by the bootloader. The timer interrupt prints a `.`
   every tick and typing on the keyboard echoes to the screen.

   If nothing boots, double-check that the machine really supports legacy
   boot; most modern laptops and virtually all UEFI-only firmware do not.

## Tests

```sh
cargo test
```

The tests do not run as ordinary host binaries. Each test binary boots the
kernel inside QEMU, runs a custom test framework in kernel space, and reports
its result through QEMU's `isa-debug-exit` device:

- exit status 33 means success (the kernel writes `0x10` to the debug-exit
  device, and QEMU turns that into exit status `(0x10 << 1) | 1 = 33`),
- a kernel panic during the run means failure.

This setup is configured in `[package.metadata.bootimage]` in `Cargo.toml`
(debug-exit device, `-serial stdio`, `-display none`, 300 second timeout) and
in `QemuExitCode` / `exit_qemu` in `src/lib.rs`.

`Cargo.toml` declares `should_panic` and `stack_overflow` explicitly with
`harness = false`; the other test files are picked up by Cargo's automatic
test discovery.

## Limitations and unfinished work

Planned, experimental or not implemented yet:

- **UEFI boot** — not supported. Booting currently requires legacy BIOS/CSM,
  which means some modern machines cannot start it at all.
- **The linked-list allocator is experimental.** Free regions are tracked and
  splitting/merging is implemented, but heap initialisation does not yet hand
  it the correct heap size, so the fixed-size block allocator stays the
  default.
- **`tests/basic_boot.rs`** is early-series scaffolding with a placeholder
  test runner; the active tests are the other three files.
- **`src/vga_buffer.txt`** is an older draft of the console implementation
  kept for reference only. It is not compiled.
- No userspace, system calls, processes or scheduler.
- No filesystem, no device drivers beyond the PIC, keyboard, VGA text mode
  and COM1.
- Single core only — no SMP or multi-core startup.
- A few unused imports and leftovers from refactoring remain in the source.

## Built with

- [Writing an OS in Rust](https://os.phil-opp.com/) by Philipp Oppermann — the
  guide this project follows.
- [`bootloader`](https://crates.io/crates/bootloader) and
  [`bootimage`](https://crates.io/crates/bootimage) — booting and disk images.
- [`x86_64`](https://crates.io/crates/x86_64) — page tables, GDT, IDT,
  interrupts and CPU control.
- [`pc-keyboard`](https://crates.io/crates/pc-keyboard) — PS/2 keyboard
  decoding.
- [`uart_16550`](https://crates.io/crates/uart_16550) — serial port.
- [`pic8259`](https://crates.io/crates/pic8259) — programmable interrupt
  controller.
- [`linked_list_allocator`](https://crates.io/crates/linked_list_allocator) —
  fallback allocator.
- [`spin`](https://crates.io/crates/spin), [`lazy_static`](https://crates.io/crates/lazy_static),
  [`volatile`](https://crates.io/crates/volatile) — synchronisation, static
  initialisation and volatile memory access.

## License

No license is declared for this repository yet. Until one is added, assume all
rights reserved.
