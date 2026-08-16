use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

// Windows 上让子进程在后台静默运行，不弹出控制台窗口
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(windows)]
pub fn silent_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

#[cfg(not(windows))]
pub fn silent_command(program: &str) -> Command {
    Command::new(program)
}

// Windows 上 npm / npx 这类 Node 命令实际是 .cmd 批处理脚本，
// Command::new 不会自动解析 .cmd，需手动补全后缀。
pub fn node_bin(name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{}.cmd", name)
    } else {
        name.to_string()
    }
}
