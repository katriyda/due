mod config;
mod notification;
mod reminder;
mod store;
mod time;
mod tray;
mod ui;

use std::rc::Rc;

slint::include_modules!();

fn main() {
    let dir = match reminder::data_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let config = match config::ensure_config_file(&dir) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("加载配置失败: {}", e);
            config::AppConfig::default()
        }
    };
    eprintln!("配置已加载: 通知方式={:?}, 默认延后={}分钟", config.notification_method, config.default_snooze_minutes);

    let mut initial_reminders = match reminder::load_reminders(&dir) {
        Ok(reminders) => reminders,
        Err(_) => {
            let examples = vec![
                reminder::Reminder::new("喝水", "每小时喝一杯水"),
                reminder::Reminder::new("站立", "久坐后站起来活动"),
            ];
            if let Err(e) = reminder::save_reminders(&dir, &examples) {
                eprintln!("保存失败: {}", e);
                std::process::exit(1);
            }
            examples
        }
    };

    // 重启时推进错过的 next_trigger
    let now = chrono::Local::now().naive_local();
    for r in &mut initial_reminders {
        time::advance_next_trigger(r, &now);
    }
    let _ = reminder::save_reminders(&dir, &initial_reminders);

    let window = MainWindow::new().unwrap();

    // 创建系统托盘
    let _tray = match tray::TrayManager::new() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("创建托盘失败: {}", e);
            panic!("托盘创建失败，无法继续: {}", e);
        }
    };

    // 构造 Store，后续所有操作通过 Store 进行
    let store = Rc::new(store::ReminderStore::new(dir, window.as_weak()));

    // 初始刷新 UI
    store.tick();

    // 添加提醒
    let store_clone = store.clone();
    window.on_add_clicked(move |title, content, interval_idx| {
        let title = title.to_string();
        if !title.is_empty() {
            store_clone.add(&title, &content.to_string(), interval_idx);
        }
    });

    // 删除提醒
    let store_clone = store.clone();
    window.on_delete_clicked(move |index| {
        if store_clone.delete(index as usize).is_ok() {
            // TODO: 需要通过 store 访问 window 来 set_selected_index(-1)
            // 暂时由 select 回调处理
        }
    });

    // 切换启用
    let store_clone = store.clone();
    window.on_toggle_enabled(move |index| {
        let _ = store_clone.toggle(index as usize);
    });

    // 选择提醒 — 填充编辑面板
    let store_clone = store.clone();
    window.on_select_reminder(move |index| {
        store_clone.select(index as usize);
    });

    // 保存编辑
    let store_clone = store.clone();
    window.on_save_clicked(
        move |index, _title, _content, _repeat_type_idx, _repeat_amount_str, _start_date, _end_date, _daily_start, _daily_end, _repeat_limit_str| {
            if let Err(e) = store_clone.save_edit(index as usize) {
                eprintln!("保存失败: {}", e);
            }
        },
    );

    // 调度器：每秒检查提醒
    let store_clone = store.clone();
    let timer = slint::Timer::default();
    timer.start(slint::TimerMode::Repeated, std::time::Duration::from_secs(1), move || {
        store_clone.tick();
    });

    window.run().unwrap();
}
