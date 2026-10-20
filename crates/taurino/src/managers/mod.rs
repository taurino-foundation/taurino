use std::sync::Arc;
pub mod menu;
pub mod shortcut;
pub mod tray;
pub mod window;

use {
    anyhow::Result,
    parking_lot::{Mutex, MutexGuard},
};

use crate::managers::{
    menu::MenuManager, shortcut::ShortcutManager, tray::TrayManager, window::WindowManager,
};

pub struct Manager {
    _menu: Arc<Mutex<MenuManager>>,
    _tray: Arc<Mutex<TrayManager>>,
    _shortcut: Arc<Mutex<ShortcutManager>>,
    _window: Arc<Mutex<WindowManager>>,
}

impl Manager {
    pub fn new(menu_manager: Arc<Mutex<MenuManager>>) -> Result<Arc<Self>> {
        let tray_manager = TrayManager::new()?;
        let shortcut_manager = ShortcutManager::new()?;
        let window_manager = WindowManager::new()?;
        let manager = Arc::new(Self {
            _menu: menu_manager.clone(),
            _tray: tray_manager.clone(),
            _shortcut: shortcut_manager.clone(),
            _window: window_manager.clone(),
        });

        {
            menu_manager.lock().bind_manager(manager.clone());
            tray_manager.lock().bind_manager(manager.clone());
            shortcut_manager.lock().bind_manager(manager.clone());
            window_manager.lock().bind_manager(manager.clone());
        }
        Ok(manager)
    }

    pub fn menu(self: &Arc<Self>) -> Result<MutexGuard<'_, MenuManager>> {
        Ok(self._menu.lock())
    }
    pub fn tray(self: &Arc<Self>) -> Result<MutexGuard<'_, TrayManager>> {
        Ok(self._tray.lock())
    }
    pub fn shortcut(self: &Arc<Self>) -> Result<MutexGuard<'_, ShortcutManager>> {
        Ok(self._shortcut.lock())
    }
    pub fn window(self: &Arc<Self>) -> Result<MutexGuard<'_, WindowManager>> {
        Ok(self._window.lock())
    }
}
