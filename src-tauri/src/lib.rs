use std::sync::atomic::{AtomicBool, Ordering};

use rust_i18n::{i18n, t};

i18n!("locales");

use log::info;
use tauri::{WindowEvent};

static QUITTING: AtomicBool = AtomicBool::new(false);

// DSH 服务监听地址与端口（集中配置，便于修改）
const DSH_HOST: &str = "127.0.0.1";
const DSH_PORT: u16 = 3080;

mod utils;

// ===== DSH 服务启停 command =====

#[tauri::command]
async fn start_dsh_service() -> Result<String, String> {
    utils::dsh::start_dsh_service_sync()
}

#[tauri::command]
async fn kill_dsh_service() -> Result<String, String> {
    match utils::dsh::kill_dsh_service_sync()? {
        None => Ok(format!("DSH service is not running on {}:{}.", DSH_HOST, DSH_PORT)),
        Some(pid) => {
            // 强杀后端口可能延迟释放，轮询核对占用是否解除
            for _ in 0..30 {
                if utils::process::is_port_available(DSH_HOST, DSH_PORT) {
                    return Ok(format!(
                        "DSH service (PID {}) killed, port {}:{} is free.",
                        pid, DSH_HOST, DSH_PORT
                    ));
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            Err(format!(
                "DSH service (PID {}) was killed, but port {}:{} is still in use.",
                pid, DSH_HOST, DSH_PORT
            ))
        }
    }
}

// 获取DSH服务url
#[tauri::command]
async fn get_dsh_url() -> Result<String, String> {
    Ok(format!("http://{}:{}", DSH_HOST, DSH_PORT))
}

#[tauri::command]
fn is_dsh_service_running() -> bool {
    utils::dsh::is_running()
}

#[tauri::command]
fn syscheck() -> bool {
    utils::syscheck::syscheck()
}

#[tauri::command]
fn syscheck_and_fix() -> bool {
    utils::syscheck::syscheck_and_fix()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 跟随系统语言：中文系统 → zh-cn，其余 → en
    let locale = sys_locale::get_locale().unwrap_or_else(|| "en".to_string());
    let locale = if locale.to_lowercase().starts_with("zh") {
        "zh-cn"
    } else {
        "en"
    };
    rust_i18n::set_locale(locale);
    utils::logger::init_logger();
    info!("{}", t!("dshdesktop.start"));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            start_dsh_service,
            kill_dsh_service,
            is_dsh_service_running,
            get_dsh_url,
            syscheck,
            syscheck_and_fix
        ])
        .setup(|app| {
            utils::tray::setup_tray(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if !QUITTING.load(Ordering::SeqCst) {
                    api.prevent_close();
                    let _ = window.hide();
                    info!("{}", t!("tray.hide"));
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
