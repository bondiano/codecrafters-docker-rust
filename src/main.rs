use std::io::Write;

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
