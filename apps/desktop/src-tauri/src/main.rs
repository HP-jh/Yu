// 屿 · 桌面宿主
// 只负责：创建透明窗口、加载 ui/、把窗口显示出来。
// 业务逻辑全在 crates/ 里，宿主不做判断。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            if let Some(win) = app.get_webview_window("main") {
                // 先显示再调整几何，避免「几何失败 → 永不显示」。
                let _ = win.show();
                let _ = win.set_always_on_top(true);
            }
            Ok(())
        })
        .on_page_load(|win, _payload| {
            let _ = win.show();
        })
        .run(tauri::generate_context!())
        .expect("failed to run yu");
}
