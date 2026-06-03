mod config;
mod reminder;
mod time;

fn main() {
    let dir = match reminder::data_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    println!("due - 提醒应用");
    println!("数据目录: {}", dir.display());

    let reminders = match reminder::load_reminders(&dir) {
        Ok(reminders) => {
            println!("已加载 {} 条提醒", reminders.len());
            reminders
        }
        Err(_) => {
            println!("暂无提醒数据，创建示例提醒...");
            let examples = vec![
                reminder::Reminder::new("喝水", "每小时喝一杯水"),
                reminder::Reminder::new("站立", "久坐后站起来活动"),
            ];
            if let Err(e) = reminder::save_reminders(&dir, &examples) {
                eprintln!("保存失败: {}", e);
                std::process::exit(1);
            }
            println!("已创建 {} 条示例提醒", examples.len());
            examples
        }
    };

    for r in &reminders {
        let status = if r.completed { "✓" } else { "○" };
        println!("  {} {} - {}", status, r.title, r.content);
    }
}
