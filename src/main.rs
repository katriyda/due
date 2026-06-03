mod config;
mod notification;
mod reminder;
mod time;
mod tray;
mod ui;

use std::cell::RefCell;
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

    let reminders = Rc::new(RefCell::new(initial_reminders));
    let window = MainWindow::new().unwrap();

    // 创建系统托盘
    let _tray = match tray::TrayManager::new() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("创建托盘失败: {}", e);
            // 托盘失败不阻止应用启动
            // 需要一个 dummy 生命周期，但 TrayManager 不支持
            // 这里用一个局部变量，_tray 会存活到 main 结束
            panic!("托盘创建失败，无法继续: {}", e);
        }
    };

    // 初始加载提醒列表
    update_reminders(&window, &reminders.borrow());

    // 添加提醒（带间隔）
    let window_weak = window.as_weak();
    let reminders_clone = reminders.clone();
    let dir_clone = dir.clone();
    window.on_add_clicked(move |title, content, interval_idx| {
        let window = window_weak.unwrap();
        let title = title.to_string();
        let content = content.to_string();
        if !title.is_empty() {
            ui::add_reminder_with_interval(&mut reminders_clone.borrow_mut(), &title, &content, interval_idx);
            update_reminders(&window, &reminders_clone.borrow());
            let _ = reminder::save_reminders(&dir_clone, &reminders_clone.borrow());
        }
    });

    // 删除提醒
    let window_weak = window.as_weak();
    let reminders_clone = reminders.clone();
    let dir_clone = dir.clone();
    window.on_delete_clicked(move |index| {
        let window = window_weak.unwrap();
        if ui::delete_reminder(&mut reminders_clone.borrow_mut(), index as usize).is_ok() {
            window.set_selected_index(-1);
            update_reminders(&window, &reminders_clone.borrow());
            let _ = reminder::save_reminders(&dir_clone, &reminders_clone.borrow());
        }
    });

    // 切换启用
    let window_weak = window.as_weak();
    let reminders_clone = reminders.clone();
    let dir_clone = dir.clone();
    window.on_toggle_enabled(move |index| {
        let window = window_weak.unwrap();
        if ui::toggle_enabled(&mut reminders_clone.borrow_mut(), index as usize).is_ok() {
            update_reminders(&window, &reminders_clone.borrow());
            let _ = reminder::save_reminders(&dir_clone, &reminders_clone.borrow());
        }
    });

    // 选择提醒 — 填充编辑面板
    let window_weak = window.as_weak();
    let reminders_clone = reminders.clone();
    window.on_select_reminder(move |index| {
        let window = window_weak.unwrap();
        let r = reminders_clone.borrow();
        let idx = index as usize;
        if idx < r.len() {
            let reminder = &r[idx];
            window.set_selected_index(index);
            window.set_edit_title(reminder.title.clone().into());
            window.set_edit_content(reminder.content.clone().into());
            window.set_repeat_type_index(ui::repeat_type_index(reminder));
            window.set_edit_start_date(ui::format_date(reminder.start_date).into());
            window.set_edit_end_date(ui::format_date(reminder.end_date).into());
            window.set_edit_daily_start(ui::format_time(reminder.daily_start).into());
            window.set_edit_daily_end(ui::format_time(reminder.daily_end).into());
            window.set_repeat_amount_value(ui::repeat_amount(reminder).into());
            window.set_repeat_limit_value(
                reminder
                    .repeat_limit
                    .map(|l| l.to_string())
                    .unwrap_or_default()
                    .into(),
            );
        }
    });

    // 保存编辑
    let window_weak = window.as_weak();
    let reminders_clone = reminders.clone();
    let dir_clone = dir.clone();
    window.on_save_clicked(
        move |index, title, content, repeat_type_idx, repeat_amount_str, start_date, end_date, daily_start, daily_end, repeat_limit_str| {
            let window = window_weak.unwrap();
            let idx = index as usize;
            let title = title.to_string();
            let content = content.to_string();
            let repeat_amount_str = repeat_amount_str.to_string();
            let start_date = start_date.to_string();
            let end_date = end_date.to_string();
            let daily_start = daily_start.to_string();
            let daily_end = daily_end.to_string();
            let repeat_limit_str = repeat_limit_str.to_string();

            let result = ui::save_reminder(
                &mut reminders_clone.borrow_mut(),
                idx,
                &title,
                &content,
                repeat_type_idx,
                &repeat_amount_str,
                &start_date,
                &end_date,
                &daily_start,
                &daily_end,
                &repeat_limit_str,
            );

            match result {
                Ok(()) => {
                    update_reminders(&window, &reminders_clone.borrow());
                    let _ = reminder::save_reminders(&dir_clone, &reminders_clone.borrow());
                }
                Err(e) => {
                    eprintln!("保存失败: {}", e);
                }
            }
        },
    );

    // 调度器：每秒检查提醒
    let reminders_clone = reminders.clone();
    let dir_clone = dir.clone();
    let window_weak = window.as_weak();
    let timer = slint::Timer::default();
    timer.start(slint::TimerMode::Repeated, std::time::Duration::from_secs(1), move || {
        let now = chrono::Local::now().naive_local();
        let mut reminders = reminders_clone.borrow_mut();
        let mut changed = false;

        for r in reminders.iter_mut() {
            if time::should_trigger_recurring(r, &now) {
                // 发送系统通知
                let _ = notification::send(&r.title, &r.content);

                // 更新提醒状态
                time::update_after_trigger(r, &now);
                changed = true;
            }
        }

        if changed {
            if let Some(window) = window_weak.upgrade() {
                update_reminders(&window, &reminders);
            }
            let _ = reminder::save_reminders(&dir_clone, &reminders);
        }
    });

    window.run().unwrap();
}

fn update_reminders(window: &MainWindow, reminders: &[reminder::Reminder]) {
    let items: Vec<ReminderItem> = ui::to_reminder_items(reminders)
        .iter()
        .map(|item| ReminderItem {
            id: item.id as i32,
            title: item.title.clone().into(),
            content: item.content.clone().into(),
            enabled: item.enabled,
            completed: item.completed,
            next_trigger: item.next_trigger.clone().unwrap_or_default().into(),
        })
        .collect();
    let model = Rc::new(slint::VecModel::from(items));
    window.set_reminders(model.into());
}
