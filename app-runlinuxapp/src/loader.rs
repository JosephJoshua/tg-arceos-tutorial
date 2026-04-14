//! ELF loader: parses ELF headers and loads PT_LOAD segments into user space.
//!
//! Uses manual ELF parsing (no external `elf` crate) and `axfs` for file I/O.

use alloc::{vec, vec::Vec};
use axfs::ROOT_FS_CONTEXT;
use axhal::paging::MappingFlags;
use axmm::AddrSpace;
use memory_addr::{MemoryAddr, VirtAddr};

// ---- Minimal ELF64 structures ----

const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
const PT_LOAD: u32 = 1;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Elf64Ehdr {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Elf64Phdr {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

const EHDR_SIZE: usize = core::mem::size_of::<Elf64Ehdr>();
const PHDR_SIZE: usize = core::mem::size_of::<Elf64Phdr>();

pub fn load_user_app(fname: &str, uspace: &mut AddrSpace) -> Result<usize, axio::Error> {
    ax_println!("app: {}", fname);
    let ctx = ROOT_FS_CONTEXT.get().expect("Root FS not initialized");
    let file = axfs::File::open(ctx, fname).map_err(|_| axio::Error::NotFound)?;

    // Read entire file into buffer (seek is not supported)
    let mut file_buf = Vec::new();
    loop {
        let mut chunk = [0u8; 4096];
        let n = file.read(&mut chunk[..])?;
        if n == 0 {
            break;
        }
        if n > 0 {
            file_buf.extend_from_slice(&chunk[..n]);
        }
    }

    ax_println!("File size: {} bytes", file_buf.len());

    // Parse ELF header
    if file_buf.len() < EHDR_SIZE {
        ax_println!("Error: file too small ({} bytes)", file_buf.len());
        return Err(axio::Error::InvalidData);
    }
    let ehdr: Elf64Ehdr = unsafe { core::ptr::read_unaligned(file_buf.as_ptr() as *const _) };

    // Validate ELF magic
    if ehdr.e_ident[..4] != ELF_MAGIC {
        ax_println!(
            "Error: ELF magic mismatch: got {:02x?}, expected {:02x?}",
            &ehdr.e_ident[..4],
            ELF_MAGIC
        );
        return Err(axio::Error::InvalidData);
    }

    let entry = ehdr.e_entry as usize;
    let phnum = ehdr.e_phnum as usize;
    let phoff = ehdr.e_phoff as usize;

    ax_println!(
        "ELF: entry={:#x}, phnum={}, phoff={:#x}",
        entry,
        phnum,
        phoff
    );

    // Read program headers
    let phdr_total = phnum * PHDR_SIZE;
    if phoff + phdr_total > file_buf.len() {
        ax_println!("Error: program headers beyond file size");
        return Err(axio::Error::InvalidData);
    }
    let phdr_buf: &[u8] = &file_buf[phoff..phoff + phdr_total];

    // Process PT_LOAD segments
    for i in 0..phnum {
        let phdr: Elf64Phdr =
            unsafe { core::ptr::read_unaligned(phdr_buf.as_ptr().add(i * PHDR_SIZE) as *const _) };

        if phdr.p_type != PT_LOAD {
            continue;
        }

        ax_println!(
            "phdr {}: offset: {:#X}=>{:#X} size: {:#X}=>{:#X}",
            i,
            phdr.p_offset,
            phdr.p_vaddr,
            phdr.p_filesz,
            phdr.p_memsz
        );

        let vaddr = VirtAddr::from(phdr.p_vaddr as usize).align_down_4k();
        let vaddr_end = VirtAddr::from((phdr.p_vaddr + phdr.p_memsz) as usize).align_up_4k();
        let map_size = vaddr_end - vaddr;

        ax_println!("Mapping {:#x} - {:#x}", vaddr, vaddr_end);

        // Map pages with auto-allocated physical pages
        uspace
            .map_alloc(
                vaddr,
                map_size,
                MappingFlags::READ
                    | MappingFlags::WRITE
                    | MappingFlags::EXECUTE
                    | MappingFlags::USER,
                true,
            )
            .map_err(|_| axio::Error::NoMemory)?;

        // Read segment data from file and write to user space
        let p_offset = phdr.p_offset as usize;
        let p_filesz = phdr.p_filesz as usize;
        let p_memsz = phdr.p_memsz as usize;

        if p_offset + p_filesz > file_buf.len() {
            ax_println!("Error: segment data beyond file size");
            return Err(axio::Error::InvalidData);
        }

        // Create data buffer, copy from file, BSS region stays zero-filled
        let mut data = vec![0; p_memsz];
        if p_filesz > 0 && p_offset + p_filesz <= file_buf.len() {
            data[..p_filesz].copy_from_slice(&file_buf[p_offset..p_offset + p_filesz]);
        }

        uspace
            .write(VirtAddr::from(phdr.p_vaddr as usize), &data)
            .map_err(|_| axio::Error::WriteZero)?;
    }

    ax_println!("Load complete, entry point: {:#x}", entry);
    Ok(entry)
}
