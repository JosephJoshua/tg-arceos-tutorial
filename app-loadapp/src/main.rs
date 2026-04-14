#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

#[cfg(feature = "axstd")]
extern crate axfs;
#[cfg(feature = "axstd")]
extern crate axio;

#[cfg_attr(feature = "axstd", unsafe(no_mangle))]
fn main() {
    #[cfg(feature = "axstd")]
    {
        #[allow(unused_imports)]
        use axio::Read;
        use std::thread;

        println!("Load app from fat-fs ...");

        let mut buf = [0u8; 64];
        if let Err(e) = load_app("/sbin/origin.bin", &mut buf) {
            panic!("Cannot load app! {:?}", e);
        }

        let worker1 = thread::spawn(move || {
            println!("worker1 checks code: ");
            for b in buf.iter().take(8) {
                print!("{:#x} ", b);
            }
            println!("\nworker1 ok!");
        });

        println!("Wait for workers to exit ...");
        let _ = worker1.join();

        println!("Load app from disk ok!");

        fn load_app(fname: &str, buf: &mut [u8]) -> Result<usize, axio::Error> {
            println!("fname: {}", fname);
            let mut file = axfs::api::File::open(fname).map_err(|_| axio::Error::NotFound)?;
            let n = file.read(buf)?;
            Ok(n)
        }
    }
    #[cfg(not(feature = "axstd"))]
    {
        println!("This application requires the 'axstd' feature for filesystem access.");
        println!("Run with: cargo xtask run [--arch <ARCH>]");
    }
}
