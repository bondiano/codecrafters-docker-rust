#[cfg(target_os = "linux")]
use libc;
use std::env::{args, set_current_dir};
use std::fs::{copy, create_dir, create_dir_all, set_permissions, File, Permissions};
use std::os::unix::fs::{chroot, PermissionsExt};
use std::process::{exit, Command, Stdio};

use anyhow::{Context, Result};
use tempfile::TempDir;

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    let args: Vec<_> = args().collect();
    let command = &args[3];
    let command_args = &args[4..];

    let exit_code = run_command(command, command_args)?;
    exit(exit_code);
}

fn run_command(command: &String, command_args: &[String]) -> Result<i32> {
    let temp_dir = tempfile::tempdir()?;

    copy_command(command, &temp_dir)?;

    create_dev_null(&temp_dir)?;

    chroot(temp_dir.path())?;

    set_current_dir("/")?;

    #[cfg(target_os = "linux")]
    unsafe {
        libc::unshare(libc::CLONE_NEWPID)
    };

    let mut command = Command::new(command)
        .args(command_args)
        .stdin(Stdio::null())
        .spawn()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;

    Ok(command.wait()?.code().unwrap_or(1))
}

fn copy_command(command: &String, temp_dir: &TempDir) -> Result<()> {
    let command_path_relative = command.trim_start_matches("/");
    let target_command = temp_dir.path().join(command_path_relative);
    let target_path = target_command.parent().unwrap();

    create_dir_all(target_path)?;
    copy(command, target_command)?;

    Ok(())
}

fn create_dev_null(temp_dir: &TempDir) -> Result<()> {
    create_dir(temp_dir.path().join("dev"))?;
    set_permissions(temp_dir.path().join("dev"), Permissions::from_mode(0o555))?;

    File::create(temp_dir.path().join("dev/null"))?;
    set_permissions(
        temp_dir.path().join("dev/null"),
        Permissions::from_mode(0o555),
    )?;

    Ok(())
}
