use std::net::TcpListener;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

use rust_i18n::{i18n, t};

i18n!("locales");

use log::{error, info, warn};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

// 为 true 表示用户通过托盘「退出」真正退出，此时 CloseRequested 不再拦截
static QUITTING: AtomicBool = AtomicBool::new(false);

#[cfg(windows)]
use std::os::windows::process::CommandExt;

// Windows 上让子进程在后台静默运行，不弹出控制台窗口
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(windows)]
fn silent_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

#[cfg(not(windows))]
fn silent_command(program: &str) -> Command {
    Command::new(program)
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// ===== DSH 服务启停（同步版，供托盘菜单等同步上下文调用）=====

// 同步启动 dsh 服务
fn start_dsh_service_sync() -> Result<String, String> {
    let host = "127.0.0.1";
    let port = 3080u16;

    if !is_port_available(host, port) {
        warn!("{}", t!("log.port_busy", port = port));
        return Err(format!("Port {} is already in use on {}.", port, host));
    }

    silent_command(npx_bin())
        .args(["--yes", "@deepseek-ai/dsh", "web"])
        .spawn()
        .map_err(|err| {
            error!("{}", t!("log.start_failed", err = err));
            format!("Failed to launch DSH service: {}", err)
        })?;

    info!("{}", t!("log.started", host = host, port = port));
    Ok(format!("DSH service started successfully on {}:{}.", host, port))
}

// 同步杀掉 dsh 服务。返回 Ok(None) 表示本来就没在跑，Ok(Some(pid)) 表示已杀掉对应进程。
fn kill_dsh_service_sync() -> Result<Option<u32>, String> {
    let host = "127.0.0.1";
    let port = 3080u16;

    // 端口空闲 = 没有 dsh 在运行
    if is_port_available(host, port) {
        info!("{}", t!("log.not_running"));
        return Ok(None);
    }

    let pid = find_pid_by_port(port)?;

    let status = silent_command("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .status()
        .map_err(|err| {
            error!("{}", t!("log.kill_failed", pid = pid, err = err));
            format!("Failed to kill DSH service (PID {}): {}", pid, err)
        })?;

    if !status.success() {
        error!("{}", t!("log.taskkill_failed", pid = pid));
        return Err(format!("taskkill failed to terminate DSH service (PID {}).", pid));
    }

    info!("{}", t!("log.stopped", pid = pid));
    Ok(Some(pid))
}

// ===== 异步命令（供前端 JS 通过 invoke 调用，内部复用同步版）=====

#[tauri::command]
async fn start_dsh_service() -> Result<String, String> {
    start_dsh_service_sync()
}

#[tauri::command]
async fn kill_dsh_service() -> Result<String, String> {
    let host = "127.0.0.1";
    let port = 3080u16;

    match kill_dsh_service_sync()? {
        None => Ok(format!("DSH service is not running on {}:{}.", host, port)),
        Some(pid) => {
            // 强杀后端口可能延迟释放，轮询核对占用是否解除
            for _ in 0..30 {
                if is_port_available(host, port) {
                    return Ok(format!(
                        "DSH service (PID {}) killed, port {}:{} is free.",
                        pid, host, port
                    ));
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            Err(format!(
                "DSH service (PID {}) was killed, but port {}:{} is still in use.",
                pid, host, port
            ))
        }
    }
}

// 
#[tauri::command]
fn is_dsh_service_running() -> bool {
    let host = "127.0.0.1";
    let port = 3080u16;
    return !is_port_available(host, port);
}

fn is_port_available(host: &str, port: u16) -> bool {
    TcpListener::bind((host, port)).is_ok()
}

// Windows 上 npx 是 npx.cmd，Command::new 不会解析 .cmd，需显式指定
fn npx_bin() -> &'static str {
    if cfg!(target_os = "windows") {
        "npx.cmd"
    } else {
        "npx"
    }
}

fn find_pid_by_port(port: u16) -> Result<u32, String> {
    let output = silent_command("netstat")
        .args(["-ano", "-p", "tcp"])
        .output()
        .map_err(|err| format!("Failed to inspect network: {}", err))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let port_suffix = format!(":{}", port);

    for line in stdout.lines() {
        // Windows netstat -ano 列: Proto, Local Address, Foreign Address, State, PID
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            continue;
        }

        let proto = parts[0];
        let local_addr = parts[1];
        let state = parts[3];
        let pid = parts[4];

        if proto.eq_ignore_ascii_case("TCP")
            && local_addr.ends_with(&port_suffix)
            && state.eq_ignore_ascii_case("LISTENING")
        {
            return pid
                .parse::<u32>()
                .map_err(|e| format!("Invalid PID in netstat output ({}): {}", pid, e));
        }
    }

    Err(format!("No process is listening on port {}.", port))
}

// 同步清理 dsh 进程（退出前调用）。on_menu_event 是同步上下文，不能用 async 的 kill_dsh_service
fn cleanup_dsh_process() {
    match find_pid_by_port(3080) {
        Ok(pid) => {
            let _ = silent_command("taskkill")
                .args(["/PID", &pid.to_string(), "/T", "/F"])
                .status();
            info!("{}", t!("log.cleaned", pid = pid));
        }
        Err(_) => {
            info!("{}", t!("log.cleanup_nothing"));
        }
    }
}

// 根据服务启停状态刷新「服务控制」菜单文字。
// MenuItem 的 set_text 内部会自行切回主线程执行，因此可在任意线程安全调用。
fn update_service_menu_text<R: tauri::Runtime>(item: &MenuItem<R>) {
    let new_title = if is_dsh_service_running() {
        t!("menu.stop_service")
    } else {
        t!("menu.start_service")
    };
    item.set_text(new_title).ok();
}

// 创建系统托盘：图标 + 右键菜单
fn setup_tray(app: &mut tauri::App) -> tauri::Result<()> {
    let show_i = MenuItem::with_id(app, "show", t!("menu.show"), true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", t!("menu.quit"), true, None::<&str>)?;
    let service_control_i =
        MenuItem::with_id(app, "service_control", t!("menu.service_control"), true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &quit_i, &service_control_i])?;

    // 给点击处理器单独克隆一份（MenuItem 可无限 clone，各副本指向同一菜单项）
    let service_control_click = service_control_i.clone();

    let tray = TrayIconBuilder::with_id("main")
        .icon(tauri::include_image!("icons/32x32.png"))
        .tooltip("DSH Desktop")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                info!("{}", t!("log.user_quit"));
                QUITTING.store(true, Ordering::SeqCst);
                cleanup_dsh_process();
                app.exit(0);
            }
            "service_control" => {
                if is_dsh_service_running() {
                    let _ = kill_dsh_service_sync();
                } else {
                    let _ = start_dsh_service_sync();
                }
                // 切换后立即刷新文字，不等下一轮 2 秒轮询
                update_service_menu_text(&service_control_click);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // 左键单击托盘图标：恢复窗口
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    // 关键：交给 state 管理器持有，否则 build 返回的句柄被 drop 后托盘图标会消失
    app.manage(tray);

    // 后台线程：每 2 秒刷新一次服务控制菜单文字
    let service_control_item = service_control_i.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(2));
        update_service_menu_text(&service_control_item);
    });

    info!("{}", t!("log.tray_created"));
    Ok(())
}

// 初始化日志：统一 [时间戳] [级别] 信息 格式，输出到控制台
fn init_logger() {
    use std::io::Write;
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .format(|buf, record| {
            let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            writeln!(buf, "[{}] [{}] {}", ts, record.level(), record.args())
        })
        .init();
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
    init_logger();
    info!("{}", t!("dshdesktop.start"));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, start_dsh_service, kill_dsh_service, is_dsh_service_running])
        .setup(|app| {
            setup_tray(app)?;
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
