use std::path::Path;

use log::{error, info, warn};

use crate::utils::terminal::node_bin;

// 判断某个命令是否存在于 PATH 中（例如 npm）
fn is_package_exist(package: &str) -> bool {
    // Windows 用 where（能匹配 npm.cmd / npm.exe / npm.bat），Unix 用 which
    #[cfg(windows)]
    let locator = "where";
    #[cfg(not(windows))]
    let locator = "which";

    crate::utils::terminal::silent_command(locator)
        .arg(package)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

// 判断某个全局 npm 包是否已安装
fn is_npm_package_installed_global(package: &str) -> bool {
    // 获取全局 node_modules 路径
    let Ok(output) = crate::utils::terminal::silent_command(&node_bin("npm"))
        .args(["root", "-g"])
        .output()
    else {
        return false;
    };
    if !output.status.success() {
        return false;
    }
    let global_root = String::from_utf8_lossy(&output.stdout);
    let global_root = global_root.trim();
    let pkg_dir = Path::new(global_root).join(package);
    pkg_dir.is_dir()
}

// 运行一条安装命令并记录结果
fn run_installer(program: &str, args: &[&str], manual_hint: &str) -> bool {
    let result = crate::utils::terminal::silent_command(program)
        .args(args)
        .status();
    match result {
        Ok(status) if status.success() => {
            info!("安装完成: {}", manual_hint);
            true
        }
        Ok(_) => {
            error!("安装失败，请手动执行: {}", manual_hint);
            false
        }
        Err(err) => {
            error!(
                "无法执行安装命令 {}: {}，请手动执行: {}",
                program, err, manual_hint
            );
            false
        }
    }
}

// 安装 Node.js（含 npm / npx）
fn install_node() -> bool {
    // Windows：不自动安装，提示用户手动安装
    if cfg!(target_os = "windows") {
        warn!("检测到未安装 Node.js。请前往 https://nodejs.org 下载安装，完成后重启本应用。");
        return false;
    }

    // Linux：用 apt 安装 npm + nodejs
    if cfg!(target_os = "linux") {
        info!("正在通过 apt 安装 npm / nodejs ...");
        return run_installer(
            "sudo",
            &["apt", "install", "-y", "npm", "nodejs"],
            "sudo apt install -y npm nodejs",
        );
    }

    // macOS：用 Homebrew 安装 node（自带 npm / npx）
    if cfg!(target_os = "macos") {
        info!("正在通过 Homebrew 安装 node ...");
        return run_installer("brew", &["install", "node"], "brew install node");
    }

    false
}

// 全局安装 @deepseek-ai/dsh
fn install_dsh() -> bool {
    info!("正在全局安装 @deepseek-ai/dsh ...");
    run_installer(
        &node_bin("npm"),
        &["install", "-g", "@deepseek-ai/dsh"],
        "npm install -g @deepseek-ai/dsh",
    )
}

pub fn syscheck() -> bool {
    is_package_exist("npm")
        && is_package_exist("npx")
        && is_npm_package_installed_global("@deepseek-ai/dsh")
}

pub fn syscheck_and_fix() -> bool {
    // 1. npm / npx 缺失 → 安装 Node.js（Windows 仅提示，需手动装）
    if !is_package_exist("npm") || !is_package_exist("npx") {
        if !install_node() {
            return false;
        }
    }

    // 2. dsh 全局包缺失 → 全局安装
    if !is_npm_package_installed_global("@deepseek-ai/dsh") {
        if !install_dsh() {
            return false;
        }
    }

    true
}
