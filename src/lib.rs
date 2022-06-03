//! # sgx-panic-backtrace
//!
//! A small library for printing out panics and backtraces inside an SGX enclave.
//!
//! ## Why
//!
//! + Get backtraces working while we wait for `backtrace-rs` to get fixed : )
//!
//! ## Usage
//!
//! Add `sgx-panic-backtrace` to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! sgx-panic-backtrace = "0.1.0"
//! ```
//!
//!
//! In the enclave, call `sgx_panic_backtrace::set_panic_hook()` in your main
//! function:
//!
//! ```rust,no_run
//! sgx_panic_backtrace::set_panic_hook();
//! ```
//!
//! If the enclave panics (and panic=abort is not turned on!) it will now print
//! out the raw backtrace frames to stdout. These include only the frame index
//! and relative frame instruction pointer offset, which you'll need to symbolize
//! outside the enclave itself.
//!
//! ```bash
//! $ cargo run --target=x86_64-fortanix-unknown-sgx
//!
//! enclave: panicked at 'foo', bar.rs:10:5
//! stack backtrace:
//!    0: 0x1b09d9
//!    1: 0x1396f6
//!    2: 0x10f4cc
//!    3: 0x48b3ef
//!    4: 0x2d540b
//!    5: 0x2d56fa
//!    6: 0x2d531d
//!    7: 0x16c681
//!    8: 0x116fd0
//!    9: 0x13410e
//! ```
//!
//! To get human readable symbol names and locations from these raw ips, you may
//! wish to use the `stack-trace-resolve` utility that comes with the Fortanix
//! EDP.
//!
//! For example:
//!
//! ```bash
//! $ ftxsgx-runner <my-enclave-bin>.sgxs | stack-trace-resolve <my-enclave-bin>
//! ```

use std::{io::Write, panic};

/// Return the base address of the currently loaded SGX enclave binary. Vendoring
/// this lets us avoid requiring the unstable `sgx_platform` feature.
///
/// This is copied from: [std::os::fortanix_sgx::mem::image_base](https://github.com/rust-lang/rust/blob/master/library/std/src/sys/sgx/abi/mem.rs#L37)
// NOTE: Do not remove inline: will result in relocation failure.
#[cfg(all(target_vendor = "fortanix", target_env = "sgx"))]
#[inline(always)]
fn image_base() -> u64 {
    use std::arch::asm;

    let base: u64;
    unsafe {
        asm!(
            // `IMAGE_BASE` is defined here:
            // [std/src/sys/sgx/abi/entry.S](https://github.com/rust-lang/rust/blob/master/library/std/src/sys/sgx/abi/entry.S#L5)
            "lea IMAGE_BASE(%rip), {}",
            lateout(reg) base,
            options(att_syntax, nostack, preserves_flags, nomem, pure),
        )
    };
    base
}

#[cfg(not(all(target_vendor = "fortanix", target_env = "sgx")))]
fn image_base() -> u64 {
    0
}

/// Trace each frame and print each relative instruction pointer offset. These
/// offsets should be symbolized these outside the enclave.
fn print_backtrace_frames() {
    println!("stack backtrace:");

    let mut frame_idx: usize = 0;
    unsafe {
        backtrace::trace_unsynchronized(|frame| {
            let base_addr = image_base() as usize;

            // we need the ip offsets relative to the binary base address.
            let ip = (frame.ip() as usize).saturating_sub(base_addr);

            println!("{frame_idx:>4}: {ip:#x}");
            frame_idx += 1;

            // TODO(phlip9): be smarter and ignore frames inside the
            // panic/backtrace code.
            // keep tracing until we run out of frames
            true
        })
    }
    println!();
}

/// Set a panic hook that will print out the panic and raw backtrace addresses
/// when the enclave panics. These addresses will need to be symbolized to human-
/// readable symbol names and locations outside the enclave with a tool like
/// `addr2line`.
pub fn set_panic_hook() {
    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // The default panic hook also doesn't print out the panic message, so
        // let's do that here.
        println!("enclave panic: {panic_info}");

        // trace the stack frames and print them out
        print_backtrace_frames();

        // enclave's about to abort. let's try to flush stdout so we get the
        // full panic message out. ignore any errors so we don't double panic.
        let _ = std::io::stdout().flush();

        // continue the default panic behaviour.
        prev_hook(panic_info);
    }));
}
