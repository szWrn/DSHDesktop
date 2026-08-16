use std::sync::atomic::Ordering;

use log::info;
use rust_i18n::t;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

use crate::utils::dsh::{cleanup_dsh_process, is_running, kill_dsh_service_sync, start_dsh_service_sync};
use crate::QUITTING;

// 根据服务启停状态刷新「服务控制」菜单文字。
// MenuItem 的 set_text 内部会自行切回主线程执行，因此可在任意线程安全调用。
fn update_service_menu_text<R: tauri::Runtime>(item: &MenuItem<R>) {
    let new_title = if is_running() {
        t!("menu.stop_service")
    } else {
        t!("menu.start_service")
    };
    item.set_text(new_title).ok();
}

// 创建系统托盘：图标 + 右键菜单
pub fn setup_tray(app: &mut tauri::App) -> tauri::Result<()> {
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
                if is_running() {
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
