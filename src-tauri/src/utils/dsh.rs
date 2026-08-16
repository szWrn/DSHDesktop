use log::{error, info, warn};
use rust_i18n::t;

use crate::utils::process::{find_pid_by_port, is_port_available, kill_process_by_pid};
use crate::utils::terminal::{node_bin, silent_command};
use crate::{DSH_HOST, DSH_PORT};

// 同步启动 dsh 服务
pub fn start_dsh_service_sync() -> Result<String, String> {
    if !is_port_available(DSH_HOST, DSH_PORT) {
        warn!("{}", t!("log.port_busy", port = DSH_PORT));
        return Err(format!("Port {} is already in use on {}.", DSH_PORT, DSH_HOST));
    }

    silent_command(&node_bin("npx"))
        .args(["--yes", "@deepseek-ai/dsh", "web"])
        .spawn()
        .map_err(|err| {
            error!("{}", t!("log.start_failed", err = err));
            format!("Failed to launch DSH service: {}", err)
        })?;

    info!("{}", t!("log.started", host = DSH_HOST, port = DSH_PORT));
    Ok(format!("DSH service started successfully on {}:{}.", DSH_HOST, DSH_PORT))
}

// 同步关闭 dsh 服务
pub fn kill_dsh_service_sync() -> Result<Option<u32>, String> {
    // 端口空闲 = 没有 dsh 在运行
    if is_port_available(DSH_HOST, DSH_PORT) {
        info!("{}", t!("log.not_running"));
        return Ok(None);
    }

    let pid = find_pid_by_port(DSH_PORT)?;

    kill_process_by_pid(pid).map_err(|err| {
        error!("{}", t!("log.kill_failed", pid = pid, err = err));
        err
    })?;

    info!("{}", t!("log.stopped", pid = pid));
    Ok(Some(pid))
}

// dsh 服务是否正在运行（端口被占用）
pub fn is_running() -> bool {
    !is_port_available(DSH_HOST, DSH_PORT)
}

// 同步清理 dsh 进程（退出前调用）。on_menu_event 是同步上下文，不能用 async 的 kill_dsh_service
pub fn cleanup_dsh_process() {
    match find_pid_by_port(DSH_PORT) {
        Ok(pid) => {
            if let Err(err) = kill_process_by_pid(pid) {
                error!("{}", t!("log.kill_failed", pid = pid, err = err));
            } else {
                info!("{}", t!("log.cleaned", pid = pid));
            }
        }
        Err(_) => {
            info!("{}", t!("log.cleanup_nothing"));
        }
    }
}
