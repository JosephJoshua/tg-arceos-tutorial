use axfs::ROOT_FS_CONTEXT;
use axhal::paging::MappingFlags;
use axmm::AddrSpace;

use crate::APP_ENTRY;

pub fn load_user_app(fname: &str, uspace: &mut AddrSpace) -> Result<(), axio::Error> {
    let mut buf = [0u8; axhal::mem::PAGE_SIZE_4K];
    let n = load_file(fname, &mut buf)?;

    // Map user code with eager allocation (populate=true)
    uspace
        .map_alloc(
            (APP_ENTRY).into(),
            axhal::mem::PAGE_SIZE_4K,
            MappingFlags::READ | MappingFlags::WRITE | MappingFlags::EXECUTE | MappingFlags::USER,
            true, // populate=true: allocate immediately
        )
        .map_err(|_| axio::Error::NoMemory)?;

    // Write the loaded data into the address space
    uspace
        .write((APP_ENTRY).into(), &buf[..n])
        .map_err(|_| axio::Error::NoMemory)?;

    ax_println!("Loaded app {} ({} bytes) at {:#x}", fname, n, APP_ENTRY);

    Ok(())
}

fn load_file(fname: &str, buf: &mut [u8]) -> Result<usize, axio::Error> {
    ax_println!("app: {}", fname);
    let ctx = ROOT_FS_CONTEXT.get().expect("Root FS not initialized");
    let file = axfs::File::open(ctx, fname).map_err(|_| axio::Error::NotFound)?;
    let n = file.read(buf)?;
    Ok(n)
}
