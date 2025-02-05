# sgx-panic-backtrace

A small library for printing out panics and backtraces inside an SGX enclave.

**ARCHIVED(2025-02-05):** backtraces are now properly relativized on latest rust std.

## Why

+ Get backtraces working while we wait for `backtrace-rs` to get fixed : )

## Usage

Add `sgx-panic-backtrace` to your `Cargo.toml`:

```toml
[dependencies]
sgx-panic-backtrace = "0.1.0"
```


In the enclave, call `sgx_panic_backtrace::set_panic_hook()` in your main
function:

```rust,no_run
sgx_panic_backtrace::set_panic_hook();
```

If the enclave panics (and panic=abort is not turned on!) it will now print
out the raw backtrace frames to stdout. These include only the frame index
and relative frame instruction pointer offset, which you'll need to symbolize
outside the enclave itself.

```bash
$ cargo run --target=x86_64-fortanix-unknown-sgx

enclave: panicked at 'foo', bar.rs:10:5
stack backtrace:
   0: 0x1b09d9
   1: 0x1396f6
   2: 0x10f4cc
   3: 0x48b3ef
   4: 0x2d540b
   5: 0x2d56fa
   6: 0x2d531d
   7: 0x16c681
   8: 0x116fd0
   9: 0x13410e
```

To get human readable symbol names and locations from these raw ips, you may
wish to use the `stack-trace-resolve` utility that comes with the Fortanix
EDP.

For example:

```bash
$ ftxsgx-runner <my-enclave-bin>.sgxs | stack-trace-resolve <my-enclave-bin>
```
