use std::process::Command;

pub mod file;
pub mod flate;

#[cfg(target_os = "windows")]
pub fn create_hidden_command(cmd: &str) -> Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let mut command = Command::new(cmd);
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

#[cfg(not(target_os = "windows"))]
pub fn create_hidden_command(cmd: &str) -> Command {
    Command::new(cmd)
}
