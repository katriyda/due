use due::reminder::{self, RepeatInterval};
use due::store::ReminderStore;

/// 跨实例持久化：Store A 添加提醒 → 丢弃 → Store B 加载 → 验证数据完整
#[test]
fn cross_instance_persistence() {
    let dir = tempfile::tempdir().unwrap();

    // Store A: 添加提醒
    {
        let store = ReminderStore::new(dir.path().to_path_buf(), slint::Weak::default());
        store.add("喝水", "每小时喝一杯水", 3); // 1小时
        store.add("站立", "久坐后站起来", 4);    // 1天
    }
    // Store A 丢弃，模拟应用关闭

    // Store B: 加载并验证
    {
        let _store = ReminderStore::new(dir.path().to_path_buf(), slint::Weak::default());
        let reminders = reminder::load_reminders(dir.path()).unwrap();

        assert_eq!(reminders.len(), 2);
        assert_eq!(reminders[0].title, "喝水");
        assert_eq!(reminders[0].content, "每小时喝一杯水");
        assert_eq!(reminders[0].repeat, Some(RepeatInterval::Hours(1)));
        assert!(reminders[0].enabled);
        assert!(reminders[0].next_trigger.is_some());

        assert_eq!(reminders[1].title, "站立");
        assert_eq!(reminders[1].repeat, Some(RepeatInterval::Days(1)));
    }
}

/// 完整 CRUD 工作流：添加 → 编辑 → 切换启用 → 删除 → 验证文件为空
#[test]
fn full_crud_workflow() {
    let dir = tempfile::tempdir().unwrap();
    let store = ReminderStore::new(dir.path().to_path_buf(), slint::Weak::default());

    // 添加
    store.add("喝水", "每小时喝一杯水", 3);
    assert_eq!(reminder::load_reminders(dir.path()).unwrap().len(), 1);

    // 编辑
    store.apply_edit_data(
        0, "新标题", "新内容", 1, "30",
        "", "", "", "", "",
    ).unwrap();
    let reminders = reminder::load_reminders(dir.path()).unwrap();
    assert_eq!(reminders[0].title, "新标题");
    assert_eq!(reminders[0].content, "新内容");
    assert_eq!(reminders[0].repeat, Some(RepeatInterval::Minutes(30)));

    // 切换启用
    store.toggle(0).unwrap();
    let reminders = reminder::load_reminders(dir.path()).unwrap();
    assert!(!reminders[0].enabled);

    // 删除
    store.delete(0).unwrap();
    let reminders = reminder::load_reminders(dir.path()).unwrap();
    assert_eq!(reminders.len(), 0);
}

/// 多提醒组合操作：添加 3 个 → 删除 1 个 → 编辑 1 个 → 切换 1 个 → 验证最终状态
#[test]
fn multi_reminder_workflow() {
    let dir = tempfile::tempdir().unwrap();
    let store = ReminderStore::new(dir.path().to_path_buf(), slint::Weak::default());

    // 添加 3 个
    store.add("喝水", "每小时", 3);   // index 0
    store.add("站立", "久坐后", 4);   // index 1
    store.add("买菜", "鸡蛋", 2);     // index 2

    // 删除中间的（站立）
    store.delete(1).unwrap();

    // 编辑第一个（喝水 → 新标题）
    store.apply_edit_data(
        0, "新标题", "新内容", 3, "7",
        "", "", "", "", "",
    ).unwrap();

    // 切换最后一个的启用状态
    store.toggle(1).unwrap();

    // 验证最终状态
    let reminders = reminder::load_reminders(dir.path()).unwrap();
    assert_eq!(reminders.len(), 2);
    assert_eq!(reminders[0].title, "新标题");
    assert_eq!(reminders[0].repeat, Some(RepeatInterval::Days(7)));
    assert!(reminders[0].enabled);
    assert_eq!(reminders[1].title, "买菜");
    assert!(!reminders[1].enabled);
}
