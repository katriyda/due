use muda::{Menu, MenuItem};
use tray_icon::TrayIcon;

/// 系统托盘管理器
pub struct TrayManager {
    _icon: TrayIcon,
    _menu: Menu,
}

impl std::fmt::Debug for TrayManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrayManager").finish()
    }
}

impl TrayManager {
    /// 创建系统托盘
    pub fn new() -> Result<Self, String> {
        let menu = Menu::new();
        let add_item = MenuItem::new("添加提醒", true, None);
        let pause_item = MenuItem::new("暂停所有", true, None);
        let quit_item = MenuItem::new("退出", true, None);

        menu.append(&add_item)
            .map_err(|e| format!("添加菜单项失败: {}", e))?;
        menu.append(&pause_item)
            .map_err(|e| format!("添加菜单项失败: {}", e))?;
        menu.append(&quit_item)
            .map_err(|e| format!("添加菜单项失败: {}", e))?;

        let icon = TrayIcon::new(tray_icon::TrayIconAttributes {
            icon: None,
            menu: Some(Box::new(menu.clone())),
            tooltip: Some("due - 提醒应用".to_string()),
            ..Default::default()
        })
        .map_err(|e| format!("创建托盘图标失败: {}", e))?;

        Ok(Self {
            _icon: icon,
            _menu: menu,
        })
    }

    /// 获取菜单项数量
    pub fn menu_item_count(&self) -> usize {
        self._menu.items().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_tray_manager() {
        let result = TrayManager::new();

        assert!(result.is_ok(), "创建托盘管理器应该成功: {:?}", result);
    }

    #[test]
    fn tray_menu_has_three_items() {
        let tray = TrayManager::new().unwrap();

        assert_eq!(tray.menu_item_count(), 3, "菜单应该有3个项");
    }
}
