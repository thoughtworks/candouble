use std::env;

fn main() -> std::io::Result<()> {
    let path = env::current_dir()?;
    println!("cargo:rustc-link-search={}/lib/PCBUSB", path.display());
    Ok(())
}
