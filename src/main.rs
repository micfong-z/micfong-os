fn main() {
    // read env variables that were set in build script
    let uefi_path = env!("UEFI_PATH");
    let bios_path = env!("BIOS_PATH");

    // choose whether to start the UEFI or BIOS image
    // TODO: note that we are only able to use BIOS booting for now
    // TODO: because APIC is not yet implemented
    let uefi = false;

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    if uefi {
        cmd.arg("-bios").arg("./src/OVMF-pure-efi.fd");
        cmd.arg("-drive")
            .arg(format!("format=raw,file={uefi_path}"));
        println!("Running UEFI image: {}", uefi_path);
        panic!("UEFI booting is not yet supported")
    } else {
        cmd.arg("-drive")
            .arg(format!("format=raw,file={bios_path}"));
        println!("Running BIOS image: {}", bios_path);
    }
    cmd.arg("-serial").arg("stdio");
    cmd.arg("-m").arg("512M");
    let Ok(mut child) = cmd.spawn() else { return () };
    match child.wait() {
        Ok(it) => it,
        Err(_) => return (),
    };
}
