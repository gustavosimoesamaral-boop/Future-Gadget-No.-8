use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let kernel = PathBuf::from(
        env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap()
    );

    let bios_path = out_dir.join("nosso_os-bios.img");

    bootloader::BiosBoot::new(&kernel)
        .create_disk_image(&bios_path)
        .unwrap();

    println!(
        "cargo:rustc-env=NOSSO_OS_BIOS={}",
        bios_path.display()
    );
}