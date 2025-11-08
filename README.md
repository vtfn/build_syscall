# build_syscall
Simple library that resolves Windows syscall numbers at compile time to create inlined zero overhead wrappers you can call directly. Because the syscall numbers are fetched at compilation, the generated binary is tied to the compilee's Windows version.

Currently supports up to 4 arguments. Only supports x86_64.

# How to use
```rust
// "syscall" is a declarative macro that generates the assembly needed for the syscall.
// NtStatus is equivalent to Result<(), NonZeroUsize>. This makes it niche optimized.
use build_syscall::{NtStatus, syscall};

// Use build_syscall to define "nt_close"
// The return type for the function definition is set to NtStatus
syscall!("NtClose", pub fn nt_close(handle: *const ()));

...

nt_close(core::ptr::null()) // Err(0xc0000008) STATUS_INVALID_HANDLE
```
```asm
mov eax, 0xf   ; directly load syscall id
xor r10d, r10d ; arg1 = null handle
syscall 
```
