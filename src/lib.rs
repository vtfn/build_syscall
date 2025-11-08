#![no_std]
#![feature(decl_macro)]
#![feature(maybe_uninit_array_assume_init)]
use core::mem::MaybeUninit;

include!(concat!(env!("OUT_DIR"), "/syscall.rs"));

pub const fn fnv1a_64(s: &'static str) -> u64 {
    const PRIME: u64 = 0x100000001b3;
    let mut hash = 0xcbf29ce484222325;

    let s = s.as_bytes();
    let mut i = 0;

    while i < s.len() {
        hash = (hash ^ s[i] as u64).wrapping_mul(PRIME);
        i += 1;
    }

    hash
}

pub type NtStatus = Result<(), ::core::num::NonZeroUsize>;

pub macro syscall {
    ($export:expr, $vis:vis fn $fun:ident($($arg_name:ident: $arg_ty:ty),* $(,)?)) => {
        syscall!(@emit $export, $vis $fun $($arg_name $arg_ty)*);
    },

    (@emit $export:expr, $vis:vis $fun:ident $($arg_name:ident $arg_ty:ty),* $(,)?) => {
        #[inline(always)]
        $vis fn $fun($($arg_name:$arg_ty),*) -> NtStatus {
            const ID: usize = const {
                let hash = fnv1a_64($export);

                let mut result = None;
                let mut i = 0;

                while i < SYSCALLS.len() {
                    if SYSCALLS[i].0 == hash {
                        result = Some(SYSCALLS[i].1);
                        break;
                    }

                    i += 1;
                }

                match result {
                    Some(id) => id,
                    None => panic!("couldn't find the syscall number"),
                }
            };

            let status: usize;

            unsafe {
                let [arg1, arg2, arg3, arg4] = {
                    let mut data: [MaybeUninit<usize>; 4] = MaybeUninit::uninit().assume_init();
                    let mut len: usize = 0;

                    $(
                        data[len] = MaybeUninit::new($arg_name as usize);
                        len += 1;
                    )*

                    data
                };

                core::arch::asm!(
                    "syscall",
                    in("r10") arg1,
                    in("rdx") arg2,
                    in("r8") arg3,
                    in("r9") arg4,
                    inlateout("rax") ID => status,
                    lateout("rcx") _,
                    lateout("r11") _,

                    options(preserves_flags),
                );

                core::mem::transmute(status)
            }
        }
    },
}
