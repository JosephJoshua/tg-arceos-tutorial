#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
extern crate axstd as std;

extern crate alloc;

#[cfg(feature = "axstd")]
#[macro_use]
extern crate axlog;

#[cfg(feature = "axstd")]
extern crate axfs;
#[cfg(feature = "axstd")]
extern crate axio;

#[cfg(feature = "axstd")]
mod loader;
#[cfg(feature = "axstd")]
mod syscall;
#[cfg(feature = "axstd")]
mod task;

#[cfg(feature = "axstd")]
use alloc::string::String;
#[cfg(feature = "axstd")]
use alloc::sync::Arc;
#[cfg(feature = "axstd")]
use axhal::paging::MappingFlags;
#[cfg(feature = "axstd")]
use axmm::AddrSpace;
#[cfg(feature = "axstd")]
use axsync::Mutex;
#[cfg(feature = "axstd")]
use memory_addr::va;

#[cfg(feature = "axstd")]
pub static USER_ASPACE: Mutex<Option<Arc<Mutex<AddrSpace>>>> = Mutex::new(None);

#[cfg(feature = "axstd")]
const USER_STACK_SIZE: usize = 0x10000;
#[cfg(feature = "axstd")]
const KERNEL_STACK_SIZE: usize = 0x40000; // 256 KiB

#[cfg_attr(feature = "axstd", unsafe(no_mangle))]
fn main() {
    #[cfg(feature = "axstd")]
    {
        // User space: [0x0, 0x40_0000_0000) — same layout as app-runlinuxapp.
        let mut uspace = axmm::new_user_aspace(va!(0x0), 0x40_0000_0000).unwrap();

        let entry = match loader::load_user_app("/sbin/mapfile", &mut uspace) {
            Ok(e) => e,
            Err(err) => panic!("Cannot load app! {:?}", err),
        };
        ax_println!("entry: {:#x}", entry);

        let ustack_top = uspace.end();
        let ustack_vaddr = ustack_top - USER_STACK_SIZE;
        ax_println!(
            "Mapping user stack: {:#x?} -> {:#x?}",
            ustack_vaddr,
            ustack_top
        );
        uspace
            .map_alloc(
                ustack_vaddr,
                USER_STACK_SIZE,
                MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER,
                true,
            )
            .unwrap();

        let stack_data = kernel_elf_parser::app_stack_region(
            &[String::from("mapfile")],
            &[],
            &[],
            ustack_top.as_usize(),
        );
        let ustack_pointer = ustack_top.as_usize() - stack_data.len();
        uspace
            .write(ustack_pointer.into(), stack_data.as_slice())
            .unwrap();

        ax_println!("New user address space: {:#x?}", uspace);

        let shared = Arc::new(Mutex::new(uspace));
        *USER_ASPACE.lock() = Some(Arc::clone(&shared));
        let user_task = task::spawn_user_task(shared, entry, ustack_pointer.into());

        let exit_code = user_task.join();
        ax_println!("monolithic kernel exit [{:?}] normally!", Some(exit_code));
    }
    #[cfg(not(feature = "axstd"))]
    {
        println!("This application requires the 'axstd' feature.");
        println!("Run with: cargo xtask run [--arch <ARCH>]");
    }
}
