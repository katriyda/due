use std::rc::Rc;
use log::{info, warn, error, debug};
use due::{config, logging, reminder, store, time, tray, MainWindow};
use slint::ComponentHandle;

fn main() {
    // 初始化日志系统（在 data_dir() 之前，确保所有操作都能记录日志）
    if let Err(e) = logging::init_logging() {
        eprintln!("日志初始化失败: {}", e);
    }

    let dir = match reminder::data_dir() {
        Ok(d) => d,
        Err(e) => {
            error!("获取数据目录失败: {}", e);
            std::process::exit(1);
        }
    };

    let config = match config::ensure_config_file(&dir) {
        Ok(c) => c,
        Err(e) => {
            warn!("加载配置失败，使用默认配置: {}", e);
            config::AppConfig::default()
        }
    };
    info!("配置已加载: 通知方式={:?}, 默认延后={}分钟", config.notification_method, config.default_snooze_minutes);

    let mut initial_reminders = match reminder::load_reminders(&dir) {
        Ok(reminders) => reminders,
        Err(_) => {
            warn!("加载提醒数据失败，使用示例数据");
            let examples = vec![
                reminder::Reminder::new("喝水", "每小时喝一杯水"),
                reminder::Reminder::new("站立", "久坐后站起来活动"),
            ];
            if let Err(e) = reminder::save_reminders(&dir, &examples) {
                error!("保存示例数据失败: {}", e);
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
            error!("创建托盘失败: {}", e);
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
            debug!("用户点击添加提醒: {}", title);
            store_clone.add(&title, &content.to_string(), interval_idx);
        }
    });

    // 删除提醒
    let store_clone = store.clone();
    window.on_delete_clicked(move |index| {
        debug!("用户点击删除提醒: index={}", index);
        let _ = store_clone.delete(index as usize);
    });

    // 切换启用
    let store_clone = store.clone();
    window.on_toggle_enabled(move |index| {
        debug!("用户点击切换启用: index={}", index);
        let _ = store_clone.toggle(index as usize);
    });

    // 选择提醒 — 填充编辑面板
    let store_clone = store.clone();
    window.on_select_reminder(move |index| {
        debug!("用户选择提醒: index={}", index);
        store_clone.select(index as usize);
    });

    // 保存编辑
    let store_clone = store.clone();
    window.on_save_clicked(
        move |index, title, content, repeat_type_idx, repeat_amount_str, start_date, end_date, daily_start, daily_end, repeat_limit_str| {
            debug!("用户点击保存编辑: index={}", index);
            if let Err(e) = store_clone.apply_edit_data(
                index as usize,
                &title.to_string(),
                &content.to_string(),
                repeat_type_idx,
                &repeat_amount_str.to_string(),
                &start_date.to_string(),
                &end_date.to_string(),
                &daily_start.to_string(),
                &daily_end.to_string(),
                &repeat_limit_str.to_string(),
            ) {
                error!("保存编辑失败: {}", e);
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
