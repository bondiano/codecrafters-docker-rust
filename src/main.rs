use libc::{c_char, chroot};
use std::{io::Write, path::Path};

use anyhow::{Context, Result};

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];
    let output = std::process::Command::new(command)
        .args(command_args)
        .output()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;

    let temp_dir = tempfile::tempdir().with_context(|| "Failed to create temp dir")?;
    let temp_dir_path = temp_dir.path();
    let tmp_dir_path_cchar = temp_dir_path
        .to_str()
        .with_context(|| "Failed to convert temp dir path to c_char")?
        .as_ptr() as *const c_char;

    let command_path = Path::new(command);

    copy_executable_into_dir(temp_dir_path, command_path)?;
    create_dev_null(temp_dir_path)?;

    unsafe {
        chroot(tmp_dir_path_cchar);
    }

    if output.status.success() {
        let std_out = std::str::from_utf8(&output.stdout)?;
        let std_err = std::str::from_utf8(&output.stderr)?;

        std::io::stdout().write_all(std_out.as_bytes())?;
        std::io::stderr().write_all(std_err.as_bytes())?;
    } else {
        let exit_code = output.status.code().unwrap_or(1);
        std::process::exit(exit_code);
    }

    Ok(())
}

fn copy_executable_into_dir(chroot_dir: &Path, executable_path: &Path) -> Result<()> {
    let executable_name = executable_path
        .file_name()
        .with_context(|| "Failed to get executable name")?
        .to_str()
        .with_context(|| "Failed to convert executable name to str")?;

    let chroot_executable_path = chroot_dir.join(executable_name);
    std::fs::copy(executable_path, chroot_executable_path)
        .with_context(|| "Failed to copy executable into chroot dir")?;

    Ok(())
}

fn create_dev_null(chroot_dir: &Path) -> Result<()> {
    let dev_null_path = chroot_dir.join("dev").join("null");
    std::fs::create_dir_all(dev_null_path)
        .with_context(|| "Failed to create /dev/null in chroot dir")?;

    Ok(())
}
