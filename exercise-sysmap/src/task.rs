//! User task spawning and trap dispatch loop.
//!
//! Runs the user-space ELF binary in a loop, dispatching syscalls to
//! the handler in `syscall.rs`.

use alloc::sync::Arc;
use axhal::uspace::{ReturnReason, UserContext};
use axmm::AddrSpace;
use axsync::Mutex;
use axtask::{AxTaskRef, TaskInner};
use memory_addr::VirtAddr;

use crate::syscall;

/// Wrapper to ensure `UserContext` is 16-byte aligned on the stack.
#[repr(C, align(16))]
struct AlignedUserContext(UserContext);

/// Spawn a user task that enters user space and handles syscalls.
pub fn spawn_user_task(
    uspace: Arc<Mutex<AddrSpace>>,
    entry: usize,
    ustack_pointer: VirtAddr,
) -> AxTaskRef {
    let page_table_root = uspace.lock().page_table_root();
    let uspace_keep = Arc::clone(&uspace);

    let mut task = TaskInner::new(
        move || {
            let _uspace = uspace_keep;

            let mut aligned_uctx = AlignedUserContext(UserContext::new(entry, ustack_pointer, 0));

            #[cfg(target_arch = "aarch64")]
            axhal::asm::enable_fp();

            ax_println!(
                "Enter user space: entry={:#x}, ustack={:#x}, kstack={:#x}",
                entry,
                ustack_pointer,
                axtask::current().kernel_stack_top().unwrap(),
            );

            loop {
                let reason = aligned_uctx.0.run();
                match reason {
                    ReturnReason::Syscall => {
                        if let Some(exit_code) = syscall::handle_syscall(&mut aligned_uctx.0) {
                            axtask::exit(exit_code as _);
                        }
                    }
                    ReturnReason::Interrupt => {}
                    _ => {
                        ax_println!("Unexpected trap from user space: {:?}", reason);
                        axtask::exit(-1);
                    }
                }
            }
        },
        "userboot".into(),
        crate::KERNEL_STACK_SIZE,
    );

    task.ctx_mut().set_page_table_root(page_table_root);

    axtask::spawn_task(task)
}
