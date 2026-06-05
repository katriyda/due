use log::{info, error};
use muda::{Menu, MenuItem};
use tray_icon::{Icon, TrayIcon};

const ICON_SIZE: u32 = 32;
const ICON_RGBA: &[u8] = include_bytes!("../assets/icon.rgba");

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

        let icon = Icon::from_rgba(ICON_RGBA.to_vec(), ICON_SIZE, ICON_SIZE)
            .map_err(|e| format!("加载托盘图标失败: {}", e))?;

        let icon = TrayIcon::new(tray_icon::TrayIconAttributes {
            icon: Some(icon),
            menu: Some(Box::new(menu.clone())),
            tooltip: Some("due - 提醒应用".to_string()),
            ..Default::default()
        })
        .map_err(|e| {
            let msg = format!("创建托盘图标失败: {}", e);
            error!("{}", msg);
            msg
        })?;

        info!("系统托盘已创建");
        Ok(Self {
            _icon: icon,
            _menu: menu,
        })
    }

    /// 获取菜单项数量
    #[cfg(test)]
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
