fn main() {
    // read env variables that were set in build script
    let uefi_path = env!("UEFI_PATH");
    let bios_path = env!("BIOS_PATH");
    
    // choose whether to start the UEFI or BIOS image
    let uefi = true;

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    if uefi {
        cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        cmd.arg("-drive").arg(format!("format=raw,file={uefi_path}"));
        println!("Running UEFI image: {}", uefi_path);
    } else {
        cmd.arg("-drive").arg(format!("format=raw,file={bios_path}"));
        println!("Running BIOS image: {}", bios_path);
    }
    cmd.arg("-serial").arg("stdio");
    let Ok(mut child) = cmd.spawn() else { return () };
    match child.wait() {
        Ok(it) => it,
        Err(_) => return (),
    };
}
