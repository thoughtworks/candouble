fn main() -> std::io::Result<()> {
    #[cfg(feature="pcan")]
    println!("cargo:rustc-link-search={}/lib/PCBUSB", std::env::current_dir()?.display());
    Ok(())
}
