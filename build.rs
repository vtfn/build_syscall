use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use std::ffi::CStr;

#[cfg(target_arch = "x86_64")]
fn get_ntdll_base() -> *const () {
    #[repr(C)]
    struct TEB {
        _reserved_0: [u8; 0x60],
        pub peb: *mut PEB,
    }

    #[repr(C)]
    struct PEB {
        _reserved_0: [u8; 0x18],
        pub ldr: *mut PEBLoaderData,
    }

    #[repr(C)]
    struct PEBLoaderData {
        _reserved_0: [u8; 0x10],
        pub in_load_order_module_list: *mut ListEntry,
    }

    #[repr(C)]
    struct ListEntry {
        pub flink: *mut ListEntry,
        pub blink: *mut ListEntry,
    }

    #[repr(C)]
    struct LoaderDataTableEntry {
        _reserved_0: [u8; 0x30],
        pub addr: *mut (),
    }

    unsafe {
        let teb: *mut TEB;

        core::arch::asm!(
            "mov {}, gs:[0x30]",
            out(reg) teb
        );

        let peb = (*teb).peb;
        let ldr = (*peb).ldr;
        let head = (*ldr).in_load_order_module_list;

        (*((*head).flink as *mut LoaderDataTableEntry)).addr
    }
}

#[cfg(target_arch = "x86_64")]
fn get_ntdll_exports() -> Vec<(&'static CStr, u32)> {
    #[repr(C)]
    pub struct ImageDosHeader {
        _reserved_0: [u8; 0x3C],
        pub e_lfanew: i32,
    }

    #[repr(C)]
    pub struct ImageNtHeaders64 {
        _reserved_0: [u8; 0x88],
        pub virtual_address: u32,
    }

    #[repr(C)]
    pub struct ImageExportDirectory {
        _reserved_0: [u8; 0x18],
        pub name_count: u32,
        pub function_addr: u32,
        pub name_addr: u32,
        pub ordinal_addr: u32,
    }

    let ntdll = get_ntdll_base();
    let mut exports = Vec::new();

    unsafe {
        let dos_header = ntdll.cast::<ImageDosHeader>();

        let nt_header = ntdll
            .byte_add((*dos_header).e_lfanew as _)
            .cast::<ImageNtHeaders64>();

        let export_dir = ntdll
            .byte_add((*nt_header).virtual_address as _)
            .cast::<ImageExportDirectory>()
            .read_unaligned();

        let name_ptr = ntdll.byte_add(export_dir.name_addr as _).cast::<u32>();
        let ordinal_ptr = ntdll.byte_add(export_dir.ordinal_addr as _).cast::<u16>();
        let fn_ptr = ntdll.byte_add(export_dir.function_addr as _).cast::<u32>();

        for i in 0..export_dir.name_count as usize {
            const PROLOGUE_BYTES: u32 = u32::from_ne_bytes([0x4C, 0x8B, 0xD1, 0xB8]);
            const SYSCALL_BYTES: u16 = u16::from_ne_bytes([0x0F, 0x05]);

            let name = CStr::from_ptr(ntdll.byte_add(*name_ptr.add(i) as usize).cast());
            let ordinal = *ordinal_ptr.add(i);
            let f = ntdll.byte_add(*fn_ptr.add(ordinal as _) as _);

            if f.cast::<u32>().read_unaligned() == PROLOGUE_BYTES
                && f.byte_add(0x12).cast::<u16>().read_unaligned() == SYSCALL_BYTES
            {
                exports.push((name, *f.byte_add(4).cast()));
            }
        }
    }

    exports
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("syscall.rs");
    let mut w = BufWriter::new(File::create(&path).unwrap());

    writeln!(w, "const SYSCALLS: &[(u64, usize)] = &[").unwrap();

    for (name, num) in get_ntdll_exports() {
        let name = name.to_str().unwrap();
        writeln!(w, "   (fnv1a_64(\"{}\"), {}),", name, num).unwrap();
    }

    writeln!(w, "];").unwrap();
}
