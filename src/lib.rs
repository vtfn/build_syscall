#![no_std]
#![feature(decl_macro)]
#![feature(maybe_uninit_array_assume_init)]
use core::arch::asm;
use core::mem::MaybeUninit;

include!(concat!(env!("OUT_DIR"), "/syscall.rs"));

const fn fnv1a_64(s: &str) -> u64 {
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

pub const fn get_id(name: &str) -> Option<usize> {
    let hash = fnv1a_64(name);

    let mut result = None;
    let mut i = 0;

    while i < SYSCALLS.len() {
        if SYSCALLS[i].0 == hash {
            result = Some(SYSCALLS[i].1);
            break;
        }

        i += 1;
    }

    result
}

pub type NtStatus = Result<(), ::core::num::NonZeroUsize>;

pub macro syscall {
    ($export:expr, $vis:vis fn $fun:ident($($arg_name:ident: $arg_ty:ty),* $(,)?)) => {
        syscall!(@emit $export, $vis $fun $($arg_name $arg_ty)*);
    },

    (@emit $export:expr, $vis:vis $fun:ident $($arg_name:ident $arg_ty:ty)* ) => {
        #[inline(always)]
        $vis fn $fun($($arg_name:$arg_ty),*) -> NtStatus {
            const ID: usize = get_id($export).expect("couldn't find the syscall number");
            let status: usize;

            unsafe {
                $(
                    let $arg_name = {
                        const _: () = assert!(
                            core::mem::size_of::<$arg_ty>() <= core::mem::size_of::<usize>(),
                            "argument size should be no bigger than the size of a register"
                        );

                        let mut reg: usize = MaybeUninit::uninit().assume_init();

                        core::ptr::write(&raw mut reg as *mut $arg_ty, $arg_name);
                        reg
                    };
                )*

                syscall!(@bind ID, status, $($arg_name)*);
                core::mem::transmute(status)
            }
        }
    },

    (@exec $id:expr, $st:ident, $asm:expr, $opts:tt, $($regs:tt)*) => {
        asm!(
            $asm,
            $($regs)*,
            inlateout("rax") $id => $st,
            lateout("rcx") _,
            lateout("r11") _,
            options $opts
        )
    },

    (@bind $id:expr, $st:ident, $r1:ident) => {
        syscall!(@exec $id, $st, "syscall", (preserves_flags, nostack), in("r10") $r1)
    },

    (@bind $id:expr, $st:ident, $r1:ident $r2:ident) => {
        syscall!(@exec $id, $st, "syscall", (preserves_flags, nostack),
            in("r10") $r1, in("rdx") $r2
        )
    },

    (@bind $id:expr, $st:ident, $r1:ident $r2:ident $r3:ident) => {
        syscall!(@exec $id, $st, "syscall", (preserves_flags, nostack),
            in("r10") $r1, in("rdx") $r2, in("r8") $r3
        )
    },

    (@bind $id:expr, $st:ident, $r1:ident $r2:ident $r3:ident $r4:ident) => {
        syscall!(@exec $id, $st, "syscall", (preserves_flags, nostack),
            in("r10") $r1, in("rdx") $r2, in("r8") $r3, in("r9") $r4
        )
    },

    (@bind $id:expr, $st:ident, $r1:ident $r2:ident $r3:ident $r4:ident $r5:ident) => {
        syscall!(@exec $id, $st,
            "sub rsp, 48
             mov qword ptr [rsp + 40], {r5}
             syscall
             add rsp, 48",
            (preserves_flags),
            r5 = in(reg) $r5,
            in("r10") $r1, in("rdx") $r2, in("r8") $r3, in("r9") $r4
        )
    },

    (@bind $id:expr, $st:ident, $r1:ident $r2:ident $r3:ident $r4:ident $r5:ident $r6:ident) => {
        syscall!(@exec $id, $st,
            "sub rsp, 64
             mov qword ptr [rsp + 40], {r5}
             mov qword ptr [rsp + 48], {r6}
             syscall
             add rsp, 64",
            (preserves_flags),
            r5 = in(reg) $r5, r6 = in(reg) $r6,
            in("r10") $r1, in("rdx") $r2, in("r8") $r3, in("r9") $r4
        )
    },

    (@bind $id:expr, $st:ident, $r1:ident $r2:ident $r3:ident $r4:ident $r5:ident $r6:ident $r7:ident) => {
        syscall!(@exec $id, $st,
            "sub rsp, 64
             mov qword ptr [rsp + 40], {r5}
             mov qword ptr [rsp + 48], {r6}
             mov qword ptr [rsp + 56], {r7}
             syscall
             add rsp, 64",
            (preserves_flags),
            r5 = in(reg) $r5, r6 = in(reg) $r6, r7 = in(reg) $r7,
            in("r10") $r1, in("rdx") $r2, in("r8") $r3, in("r9") $r4
        )
    },

    (@bind $id:expr, $st:ident, $r1:ident $r2:ident $r3:ident $r4:ident $r5:ident $r6:ident $r7:ident $r8:ident) => {
        syscall!(@exec $id, $st,
            "sub rsp, 80
             mov qword ptr [rsp + 40], {r5}
             mov qword ptr [rsp + 48], {r6}
             mov qword ptr [rsp + 56], {r7}
             mov qword ptr [rsp + 64], {r8}
             syscall
             add rsp, 80",
            (preserves_flags),
            r5 = in(reg) $r5, r6 = in(reg) $r6, r7 = in(reg) $r7, r8 = in(reg) $r8,
            in("r10") $r1, in("rdx") $r2, in("r8") $r3, in("r9") $r4
        )
    },

    (@bind $id:expr, $st:ident, $r1:ident $r2:ident $r3:ident $r4:ident $r5:ident $r6:ident $r7:ident $r8:ident $r9:ident) => {
        syscall!(@exec $id, $st,
            "sub rsp, 80
             mov qword ptr [rsp + 40], {r5}
             mov qword ptr [rsp + 48], {r6}
             mov qword ptr [rsp + 56], {r7}
             mov qword ptr [rsp + 64], {r8}
             mov qword ptr [rsp + 72], {r9}
             syscall
             add rsp, 80",
            (preserves_flags),
            r5 = in(reg) $r5, r6 = in(reg) $r6, r7 = in(reg) $r7, r8 = in(reg) $r8, r9 = in(reg) $r9,
            in("r10") $r1, in("rdx") $r2, in("r8") $r3, in("r9") $r4
        )
    },

    (@bind $id:expr, $st:ident, $r1:ident $r2:ident $r3:ident $r4:ident $r5:ident $r6:ident $r7:ident $r8:ident $r9:ident $r10:ident) => {
        syscall!(@exec $id, $st,
            "sub rsp, 96
             mov qword ptr [rsp + 40], {r5}
             mov qword ptr [rsp + 48], {r6}
             mov qword ptr [rsp + 56], {r7}
             mov qword ptr [rsp + 64], {r8}
             mov qword ptr [rsp + 72], {r9}
             mov qword ptr [rsp + 80], {r10}
             syscall
             add rsp, 96",
            (preserves_flags),
            r5 = in(reg) $r5, r6 = in(reg) $r6, r7 = in(reg) $r7, r8 = in(reg) $r8, r9 = in(reg) $r9, r10 = in(reg) $r10,
            in("r10") $r1, in("rdx") $r2, in("r8") $r3, in("r9") $r4
        )
    },

    (@bind $id:expr, $st:ident, $r1:ident $r2:ident $r3:ident $r4:ident $r5:ident $r6:ident $r7:ident $r8:ident $r9:ident $r10:ident $r11:ident) => {
        syscall!(@exec $id, $st,
            "sub rsp, 96
             mov qword ptr [rsp + 40], {r5}
             mov qword ptr [rsp + 48], {r6}
             mov qword ptr [rsp + 56], {r7}
             mov qword ptr [rsp + 64], {r8}
             mov qword ptr [rsp + 72], {r9}
             mov qword ptr [rsp + 80], {r10}
             mov qword ptr [rsp + 88], {r11}
             syscall
             add rsp, 96",
            (preserves_flags),
            r5 = in(reg) $r5, r6 = in(reg) $r6, r7 = in(reg) $r7, r8 = in(reg) $r8, r9 = in(reg) $r9, r10 = in(reg) $r10, r11 = in(reg) $r11,
            in("r10") $r1, in("rdx") $r2, in("r8") $r3, in("r9") $r4
        )
    },
}
