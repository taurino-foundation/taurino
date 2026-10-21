use crate::managers::Manager;
use anyhow::Result;
use parking_lot::Mutex;
use std::{collections::HashMap, sync::Arc};
use taurino_native::{
    config::Theme,
    menu::{Menu, MenuId, RawWindow, WindowMenu},
};

// -----------------------------------------------------------------------------
// MenuManager
//
// Owns every live menu known to the application.
//
// A menu is either:
//
//   * application-wide
//       macOS uses one native menu bar for the entire application.
//
//   * per-window
//       Windows and desktop Unix platforms may initialize a separate native
//       menu for each window.
//
// `menus` is the authoritative stash of known menu objects and is also used by
// the Windows accelerator/message hook.
// -----------------------------------------------------------------------------

pub struct MenuManager {
    manager: Option<Arc<Manager>>,
    menus: HashMap<MenuId, Menu>,
    app_wide_menu: Option<Menu>,
}

impl MenuManager {
    pub fn new() -> Result<Arc<Mutex<Self>>> {
        Ok(Arc::new(Mutex::new(Self {
            manager: None,
            menus: HashMap::new(),
            app_wide_menu: None,
        })))
    }
    pub(crate) fn bind_manager(&mut self, manager: Arc<Manager>) {
        self.manager = Some(manager)
    }
    // -------------------------------------------------------------------------
    // Read access
    // -------------------------------------------------------------------------

    pub fn menus(&self) -> impl Iterator<Item = (&MenuId, &Menu)> {
        self.menus.iter()
    }

    pub fn get_menu(&self, id: &MenuId) -> Option<&Menu> {
        self.menus.get(id)
    }

    pub fn app_wide_menu(&self) -> Option<&Menu> {
        self.app_wide_menu.as_ref()
    }

    pub fn is_menu_in_use(&self, id: &MenuId) -> bool {
        self.menus.contains_key(id)
    }

    // -------------------------------------------------------------------------
    // Mutation
    // -------------------------------------------------------------------------

    pub fn update_menu(&mut self, menu: Menu) -> Option<Menu> {
        self.menus.insert(menu.id().clone(), menu)
    }

    pub fn insert_menu_into_stash(&mut self, menu: &Menu) {
        self.menus
            .entry(menu.id().clone())
            .or_insert_with(|| menu.clone());
    }

    pub fn set_app_wide_menu(&mut self, menu: Menu) -> Option<Menu> {
        let previous = self.app_wide_menu.replace(menu.clone());

        if let Some(previous) = &previous {
            self.menus.remove(previous.id());
        }

        self.insert_menu_into_stash(&menu);

        previous
    }

    pub fn remove_app_wide_menu(&mut self) -> Option<Menu> {
        let menu = self.app_wide_menu.take()?;
        let id = menu.id().clone();

        self.menus.remove(&id);

        Some(menu)
    }

    pub fn remove_menu_if_not_app_wide(&mut self, id: &MenuId) -> Option<Menu> {
        if self
            .app_wide_menu
            .as_ref()
            .is_some_and(|menu| menu.id() == id)
        {
            return None;
        }

        self.menus.remove(id)
    }

    pub fn clear(&mut self) {
        self.app_wide_menu = None;
        self.menus.clear();
    }

    // -------------------------------------------------------------------------
    // Window menu creation
    //
    // Called after the native Tao window exists, because platform-specific
    // initialization may need its native window handle.
    //
    // Contract:
    //
    // macOS:
    //   * If an application-wide menu already exists, return that menu.
    //   * Otherwise, use the explicit builder menu when supplied.
    //   * If no explicit menu exists, create an empty menu.
    //   * Register the resulting menu as the application-wide menu and
    //     initialize it for NSApp.
    //
    // Windows:
    //   * Use the explicit builder menu when supplied.
    //   * Otherwise create an empty menu.
    //   * Register it in the manager.
    //   * Initialize it for this window's HWND and theme.
    //
    // Linux / BSD:
    //   * Use the explicit builder menu when supplied.
    //   * Otherwise create an empty menu.
    //   * Register it in the manager.
    //   * GTK-specific attachment may happen later when the final container
    //     hierarchy is available.
    // -------------------------------------------------------------------------

    pub fn create_window_menu(
        &mut self,
        raw: RawWindow<'_>,
        theme: Theme,
        explicit_menu: Option<Menu>,
    ) -> anyhow::Result<WindowMenu> {
        // ---------------------------------------------------------------------
        // macOS: exactly one menu bar for the application
        // ---------------------------------------------------------------------

        #[cfg(target_os = "macos")]
        {
            // Once an application-wide menu exists, reuse it.
            if let Some(menu) = self.app_wide_menu.clone() {
                self.insert_menu_into_stash(&menu);

                let _ = (raw, theme, explicit_menu);

                // Required: early return.
                return Ok(WindowMenu {
                    is_app_wide: true,
                    menu,
                });
            }

            // Initialize the application-wide menu only once.
            let menu = match explicit_menu {
                Some(menu) => menu,
                None => Menu::new()?,
            };

            self.set_app_wide_menu(menu.clone());

            menu.init_for_nsapp();

            let _ = (raw, theme);

            // Final expression: no return required.
            Ok(WindowMenu {
                is_app_wide: true,
                menu,
            })
        }

        // Keep your existing non-macOS implementation here.

        // ---------------------------------------------------------------------
        // Windows / Linux / BSD: menu belongs to this window
        // ---------------------------------------------------------------------

        #[cfg(not(target_os = "macos"))]
        {
            // Crucially, use the explicit WindowBuilder menu instead of merely
            // registering it and then creating a different empty menu.
            let menu = match explicit_menu {
                Some(menu) => menu,
                None => Menu::new()?,
            };

            self.insert_menu_into_stash(&menu);

            // -----------------------------------------------------------------
            // Windows native initialization
            // -----------------------------------------------------------------

            #[cfg(windows)]
            {
                use taurino_native::utils::map_to_menu_theme;

                let muda_theme = map_to_menu_theme(theme);

                unsafe {
                    menu.init_for_hwnd_with_theme(raw.hwnd, muda_theme)?;
                }
            }

            // -----------------------------------------------------------------
            // Linux / BSD
            // -----------------------------------------------------------------

            #[cfg(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            {
                // The final GTK container may only become available after the
                // webview/container hierarchy has been assembled. Keep these
                // handles available through RawWindow, while actual GTK menu
                // attachment may be performed by the corresponding caller.
                let _ = (raw.gtk_window, raw.default_vbox, theme);
            }

            // Other non-macOS platforms currently require no native menu
            // initialization here.
            #[cfg(not(any(
                windows,
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            )))]
            {
                let _ = (raw, theme);
            }

            Ok(WindowMenu {
                is_app_wide: false,
                menu,
            })
        }
    }

    // -------------------------------------------------------------------------
    // Windows message hook
    // -------------------------------------------------------------------------

    #[cfg(windows)]
    pub fn install_msg_hook(
        manager: Arc<Mutex<Self>>,
    ) -> Box<dyn FnMut(*const std::ffi::c_void) -> bool + 'static> {
        Box::new(move |msg| {
            use taurino_native::windows_api::Win32::UI::WindowsAndMessaging::{
                HACCEL, MSG, TranslateAcceleratorW,
            };

            unsafe {
                let msg = msg as *const MSG;

                if msg.is_null() {
                    return false;
                }

                let manager = manager.lock();

                for menu in manager.menus.values() {
                    let translated =
                        TranslateAcceleratorW((*msg).hwnd, HACCEL(menu.inner().haccel() as _), msg);

                    if translated == 1 {
                        return true;
                    }
                }

                false
            }
        })
    }
}

unsafe impl Send for MenuManager {}
unsafe impl Sync for MenuManager {}
