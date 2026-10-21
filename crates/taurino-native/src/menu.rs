use dpi::Position;
use muda::ContextMenu as MudaContextMenu;
pub use muda::MenuId;
use serde::Deserialize;
use std::sync::Arc;
use tao::window::Window;

#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "linux",
    target_os = "netbsd",
    target_os = "openbsd"
))]
use gtk;

use anyhow::Result;

use crate::{
    image::Image,
    menu::sealed::{ContextMenuBase, IsMenuItemBase},
};

pub struct MenuItemBuilder {
    id: Option<MenuId>,
    text: String,
    enabled: bool,
    accelerator: Option<String>,
}

impl MenuItemBuilder {
    pub fn new<S: AsRef<str>>(text: S) -> Self {
        Self {
            id: None,
            text: text.as_ref().into(),
            enabled: true,
            accelerator: None,
        }
    }

    pub fn with_id<I: Into<MenuId>, S: AsRef<str>>(id: I, text: S) -> Self {
        Self {
            id: Some(id.into()),
            text: text.as_ref().into(),
            enabled: true,
            accelerator: None,
        }
    }

    pub fn id<I: Into<MenuId>>(mut self, id: I) -> Self {
        self.id = Some(id.into());
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn accelerator<S: AsRef<str>>(mut self, accelerator: S) -> Self {
        self.accelerator = Some(accelerator.as_ref().into());
        self
    }

    pub fn build(self) -> anyhow::Result<MenuItem> {
        match self.id {
            Some(id) => MenuItem::with_id(id, self.text, self.enabled, self.accelerator),
            None => MenuItem::new(self.text, self.enabled, self.accelerator),
        }
    }
}

pub struct CheckMenuItemBuilder {
    id: Option<MenuId>,
    text: String,
    enabled: bool,
    checked: bool,
    accelerator: Option<String>,
}

impl CheckMenuItemBuilder {
    pub fn new<S: AsRef<str>>(text: S) -> Self {
        Self {
            id: None,
            text: text.as_ref().into(),
            enabled: true,
            checked: true,
            accelerator: None,
        }
    }

    pub fn with_id<I: Into<MenuId>, S: AsRef<str>>(id: I, text: S) -> Self {
        Self {
            id: Some(id.into()),
            text: text.as_ref().into(),
            enabled: true,
            checked: true,
            accelerator: None,
        }
    }

    pub fn id<I: Into<MenuId>>(mut self, id: I) -> Self {
        self.id = Some(id.into());
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }
    pub fn accelerator<S: AsRef<str>>(mut self, accelerator: S) -> Self {
        self.accelerator = Some(accelerator.as_ref().into());
        self
    }

    pub fn build(self) -> anyhow::Result<CheckMenuItem> {
        match self.id {
            Some(id) => {
                CheckMenuItem::with_id(id, self.text, self.enabled, self.checked, self.accelerator)
            }
            None => CheckMenuItem::new(self.text, self.enabled, self.checked, self.accelerator),
        }
    }
}

pub struct IconMenuItemBuilder<'a> {
    id: Option<MenuId>,
    text: String,
    enabled: bool,
    icon: Option<Image<'a>>,
    native_icon: Option<NativeIcon>,
    accelerator: Option<String>,
}

impl<'a> IconMenuItemBuilder<'a> {
    pub fn new<S: AsRef<str>>(text: S) -> Self {
        Self {
            id: None,
            text: text.as_ref().into(),
            enabled: true,
            icon: None,
            native_icon: None,
            accelerator: None,
        }
    }

    pub fn with_id<I: Into<MenuId>, S: AsRef<str>>(id: I, text: S) -> Self {
        Self {
            id: Some(id.into()),
            text: text.as_ref().into(),
            enabled: true,
            icon: None,
            native_icon: None,
            accelerator: None,
        }
    }

    pub fn id<I: Into<MenuId>>(mut self, id: I) -> Self {
        self.id = Some(id.into());
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn accelerator<S: AsRef<str>>(mut self, accelerator: S) -> Self {
        self.accelerator = Some(accelerator.as_ref().into());
        self
    }

    pub fn icon(mut self, icon: Image<'a>) -> Self {
        self.icon = Some(icon);
        self.native_icon = None;
        self
    }

    pub fn native_icon(mut self, icon: NativeIcon) -> Self {
        self.native_icon = Some(icon);
        self.icon = None;
        self
    }

    pub fn build(self) -> anyhow::Result<IconMenuItem> {
        match (self.id, self.icon, self.native_icon) {
            (Some(id), Some(icon), _) => {
                IconMenuItem::with_id(id, self.text, self.enabled, Some(icon), self.accelerator)
            }
            (None, Some(icon), _) => {
                IconMenuItem::new(self.text, self.enabled, Some(icon), self.accelerator)
            }
            (Some(id), None, native) => IconMenuItem::with_id_and_native_icon(
                id,
                self.text,
                self.enabled,
                native,
                self.accelerator,
            ),
            (None, None, native) => {
                IconMenuItem::with_native_icon(self.text, self.enabled, native, self.accelerator)
            }
        }
    }
}

// -----------------------------------------------------------------------------
// fluent MenuBuilder / SubmenuBuilder
// -----------------------------------------------------------------------------

pub struct MenuBuilder {
    id: Option<MenuId>,
    items: Vec<anyhow::Result<MenuItemKind>>,
}

impl MenuBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            items: Vec::new(),
        }
    }

    pub fn with_id<I: Into<MenuId>>(id: I) -> Self {
        Self {
            id: Some(id.into()),
            items: Vec::new(),
        }
    }

    pub fn id<I: Into<MenuId>>(mut self, id: I) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn item(mut self, item: &dyn IsMenuItem) -> Self {
        self.items.push(Ok(item.kind()));
        self
    }

    pub fn items(mut self, items: &[&dyn IsMenuItem]) -> Self {
        for item in items {
            self = self.item(*item);
        }
        self
    }

    pub fn text<I: Into<MenuId>, S: AsRef<str>>(mut self, id: I, text: S) -> Self {
        self.items
            .push(MenuItem::with_id(id, text, true, None::<&str>).map(MenuItemKind::MenuItem));
        self
    }

    pub fn check<I: Into<MenuId>, S: AsRef<str>>(mut self, id: I, text: S) -> Self {
        self.items.push(
            CheckMenuItem::with_id(id, text, true, true, None::<&str>).map(MenuItemKind::Check),
        );
        self
    }

    pub fn icon<I: Into<MenuId>, S: AsRef<str>>(mut self, id: I, text: S, icon: Image<'_>) -> Self {
        self.items.push(
            IconMenuItem::with_id(id, text, true, Some(icon), None::<&str>).map(MenuItemKind::Icon),
        );
        self
    }

    pub fn native_icon<I: Into<MenuId>, S: AsRef<str>>(
        mut self,
        id: I,
        text: S,
        icon: NativeIcon,
    ) -> Self {
        self.items.push(
            IconMenuItem::with_id_and_native_icon(id, text, true, Some(icon), None::<&str>)
                .map(MenuItemKind::Icon),
        );
        self
    }

    pub fn separator(mut self) -> Self {
        self.items
            .push(PredefinedMenuItem::separator().map(MenuItemKind::Predefined));
        self
    }
    pub fn copy(mut self) -> Self {
        self.items
            .push(PredefinedMenuItem::copy(None).map(MenuItemKind::Predefined));
        self
    }
    pub fn cut(mut self) -> Self {
        self.items
            .push(PredefinedMenuItem::cut(None).map(MenuItemKind::Predefined));
        self
    }
    pub fn paste(mut self) -> Self {
        self.items
            .push(PredefinedMenuItem::paste(None).map(MenuItemKind::Predefined));
        self
    }
    pub fn select_all(mut self) -> Self {
        self.items
            .push(PredefinedMenuItem::select_all(None).map(MenuItemKind::Predefined));
        self
    }
    pub fn undo(mut self) -> Self {
        self.items
            .push(PredefinedMenuItem::undo(None).map(MenuItemKind::Predefined));
        self
    }
    pub fn redo(mut self) -> Self {
        self.items
            .push(PredefinedMenuItem::redo(None).map(MenuItemKind::Predefined));
        self
    }
    pub fn minimize(mut self) -> Self {
        self.items
            .push(PredefinedMenuItem::minimize(None).map(MenuItemKind::Predefined));
        self
    }
    pub fn maximize(mut self) -> Self {
        self.items
            .push(PredefinedMenuItem::maximize(None).map(MenuItemKind::Predefined));
        self
    }
    pub fn fullscreen(mut self) -> Self {
        self.items
            .push(PredefinedMenuItem::fullscreen(None).map(MenuItemKind::Predefined));
        self
    }
    pub fn hide(mut self) -> Self {
        self.items
            .push(PredefinedMenuItem::hide(None).map(MenuItemKind::Predefined));
        self
    }
    pub fn hide_others(mut self) -> Self {
        self.items
            .push(PredefinedMenuItem::hide_others(None).map(MenuItemKind::Predefined));
        self
    }
    pub fn show_all(mut self) -> Self {
        self.items
            .push(PredefinedMenuItem::show_all(None).map(MenuItemKind::Predefined));
        self
    }
    pub fn close_window(mut self) -> Self {
        self.items
            .push(PredefinedMenuItem::close_window(None).map(MenuItemKind::Predefined));
        self
    }
    /*   pub fn quit(mut self) -> Self {
      self
        .items
        .push(PredefinedMenuItem::quit(None).map(MenuItemKind::Predefined));
      self
    } */
    pub fn services(mut self) -> Self {
        self.items
            .push(PredefinedMenuItem::services(None).map(MenuItemKind::Predefined));
        self
    }
    pub fn bring_all_to_front(mut self) -> Self {
        self.items
            .push(PredefinedMenuItem::bring_all_to_front(None).map(MenuItemKind::Predefined));
        self
    }

    pub fn about(mut self, metadata: Option<AboutMetadata<'_>>) -> Self {
        self.items
            .push(PredefinedMenuItem::about(None, metadata).map(MenuItemKind::Predefined));
        self
    }

    pub fn build(self) -> anyhow::Result<Menu> {
        let menu = match self.id {
            Some(id) => Menu::with_id(id)?,
            None => Menu::new()?,
        };
        for item in self.items {
            menu.append(&item?)?;
        }
        Ok(menu)
    }
}

impl Default for MenuBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SubmenuBuilder<'a> {
    id: Option<MenuId>,
    text: String,
    enabled: bool,
    items: Vec<anyhow::Result<MenuItemKind>>,
    icon: Option<Image<'a>>,
    native_icon: Option<NativeIcon>,
}

impl<'a> SubmenuBuilder<'a> {
    pub fn new<S: AsRef<str>>(text: S) -> Self {
        Self {
            id: None,
            text: text.as_ref().into(),
            enabled: true,
            items: Vec::new(),
            icon: None,
            native_icon: None,
        }
    }

    pub fn with_id<I: Into<MenuId>, S: AsRef<str>>(id: I, text: S) -> Self {
        Self {
            id: Some(id.into()),
            text: text.as_ref().into(),
            enabled: true,
            items: Vec::new(),
            icon: None,
            native_icon: None,
        }
    }

    pub fn id<I: Into<MenuId>>(mut self, id: I) -> Self {
        self.id = Some(id.into());
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn submenu_icon(mut self, icon: Image<'a>) -> Self {
        self.icon = Some(icon);
        self.native_icon = None;
        self
    }

    pub fn submenu_native_icon(mut self, icon: NativeIcon) -> Self {
        self.native_icon = Some(icon);
        self.icon = None;
        self
    }

    pub fn item(mut self, item: &dyn IsMenuItem) -> Self {
        self.items.push(Ok(item.kind()));
        self
    }

    pub fn items(mut self, items: &[&dyn IsMenuItem]) -> Self {
        for item in items {
            self = self.item(*item);
        }
        self
    }

    pub fn text_item<I: Into<MenuId>, S: AsRef<str>>(mut self, id: I, text: S) -> Self {
        self.items
            .push(MenuItem::with_id(id, text, true, None::<&str>).map(MenuItemKind::MenuItem));
        self
    }

    pub fn check<I: Into<MenuId>, S: AsRef<str>>(mut self, id: I, text: S) -> Self {
        self.items.push(
            CheckMenuItem::with_id(id, text, true, true, None::<&str>).map(MenuItemKind::Check),
        );
        self
    }

    pub fn separator(mut self) -> Self {
        self.items
            .push(PredefinedMenuItem::separator().map(MenuItemKind::Predefined));
        self
    }

    pub fn build(self) -> anyhow::Result<Submenu> {
        let submenu = match (self.id, self.icon, self.native_icon) {
            (Some(id), Some(icon), _) => {
                Submenu::with_id_and_icon(id, self.text, self.enabled, Some(icon))?
            }
            (None, Some(icon), _) => Submenu::new_with_icon(self.text, self.enabled, Some(icon))?,
            (Some(id), None, Some(icon)) => {
                Submenu::with_id_and_native_icon(id, self.text, self.enabled, Some(icon))?
            }
            (None, None, Some(icon)) => {
                Submenu::new_with_native_icon(self.text, self.enabled, Some(icon))?
            }
            (Some(id), None, None) => Submenu::with_id(id, self.text, self.enabled)?,
            (None, None, None) => Submenu::new(self.text, self.enabled)?,
        };
        for item in self.items {
            submenu.append(&item?)?;
        }
        Ok(submenu)
    }
}

/// A helper trait with methods to help creating a context menu.
///
/// # Safety
///
/// This trait is ONLY meant to be implemented internally by the crate.
pub trait ContextMenu: sealed::ContextMenuBase + Send + Sync {
    /// Get the popup [`HMENU`] for this menu.
    ///
    /// The returned [`HMENU`] is valid as long as the [`ContextMenu`] is.
    ///
    /// [`HMENU`]: https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#HMENU
    #[cfg(windows)]
    #[cfg_attr(docsrs, doc(cfg(windows)))]
    fn hpopupmenu(&self) -> anyhow::Result<isize>;

    /// Popup this menu as a context menu on the specified window at the cursor position.
    fn popup(&self, window: &Window) -> anyhow::Result<()>;

    /// Popup this menu as a context menu on the specified window at the specified position.
    ///
    /// The position is relative to the window's top-left corner.
    fn popup_at<P: Into<Position>>(&self, window: &Window, position: P) -> anyhow::Result<()>;
}

pub(crate) mod sealed {
    use dpi::Position;
    use tao::window::Window;

    pub trait IsMenuItemBase {
        fn inner_muda(&self) -> &dyn muda::IsMenuItem;
    }

    pub trait ContextMenuBase {
        fn inner_context(&self) -> &dyn muda::ContextMenu;
        fn inner_context_owned(&self) -> Box<dyn muda::ContextMenu>;
        fn popup_inner<P: Into<Position>>(
            &self,
            window: &Window,
            position: Option<P>,
        ) -> anyhow::Result<()>;
    }
}

fn show_context_menu(
    menu: &dyn MudaContextMenu,
    window: &Window,
    position: Option<Position>,
) -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        use tao::platform::windows::WindowExtWindows;

        unsafe {
            menu.show_context_menu_for_hwnd(window.hwnd(), position);
        }
    }

    #[cfg(target_os = "macos")]
    {
        use tao::platform::macos::WindowExtMacOS;

        unsafe {
            menu.show_context_menu_for_nsview(window.ns_view() as _, position);
        }
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    {
        use tao::platform::unix::WindowExtUnix;

        menu.show_context_menu_for_gtk_window(window.gtk_window().as_ref(), position);
    }

    Ok(())
}

impl ContextMenu for Menu {
    #[cfg(windows)]
    fn hpopupmenu(&self) -> anyhow::Result<isize> {
        Ok(self.0.inner.hpopupmenu())
    }

    fn popup(&self, window: &Window) -> anyhow::Result<()> {
        self.popup_inner(window, None::<Position>)
    }

    fn popup_at<P: Into<Position>>(&self, window: &Window, position: P) -> anyhow::Result<()> {
        self.popup_inner(window, Some(position))
    }
}

impl sealed::ContextMenuBase for Menu {
    fn inner_context(&self) -> &dyn muda::ContextMenu {
        &self.0.inner
    }

    fn inner_context_owned(&self) -> Box<dyn muda::ContextMenu> {
        Box::new(self.0.inner.clone())
    }

    fn popup_inner<P: Into<Position>>(
        &self,
        window: &Window,
        position: Option<P>,
    ) -> anyhow::Result<()> {
        show_context_menu(&self.0.inner, window, position.map(Into::into))
    }
}

impl ContextMenu for Submenu {
    #[cfg(windows)]
    fn hpopupmenu(&self) -> anyhow::Result<isize> {
        Ok(self.0.inner.hpopupmenu())
    }

    fn popup(&self, window: &Window) -> anyhow::Result<()> {
        self.popup_inner(window, None::<Position>)
    }

    fn popup_at<P: Into<Position>>(&self, window: &Window, position: P) -> anyhow::Result<()> {
        self.popup_inner(window, Some(position))
    }
}

impl sealed::ContextMenuBase for Submenu {
    fn inner_context(&self) -> &dyn muda::ContextMenu {
        &self.0.inner
    }

    fn inner_context_owned(&self) -> Box<dyn muda::ContextMenu> {
        Box::new(self.0.inner.clone())
    }

    fn popup_inner<P: Into<Position>>(
        &self,
        window: &Window,
        position: Option<P>,
    ) -> anyhow::Result<()> {
        show_context_menu(&self.0.inner, window, position.map(Into::into))
    }
}

// -----------------------------------------------------------------------------
// wrappers
// -----------------------------------------------------------------------------

macro_rules! gen_wrapper {
    ($ty:ident, $inner:ident) => {
        pub(crate) struct $inner {
            pub(crate) inner: muda::$ty,
        }

        impl $inner {
            #[inline]
            pub(crate) fn new(inner: muda::$ty) -> Self {
                Self { inner }
            }
        }

        // Kept only because crate::Resource requires Send + Sync.
        // This relies on the caller's single-thread invariant.
        unsafe impl Send for $inner {}
        unsafe impl Sync for $inner {}

        pub struct $ty(pub(crate) Arc<$inner>);

        impl Clone for $ty {
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
        }

        impl crate::resources::Resource for $ty {}
    };
}

gen_wrapper!(Menu, MenuInner);
gen_wrapper!(MenuItem, MenuItemInner);
gen_wrapper!(Submenu, SubmenuInner);
gen_wrapper!(PredefinedMenuItem, PredefinedMenuItemInner);
gen_wrapper!(CheckMenuItem, CheckMenuItemInner);
gen_wrapper!(IconMenuItem, IconMenuItemInner);

// -----------------------------------------------------------------------------
// generic menu-item facade
// -----------------------------------------------------------------------------

pub enum MenuItemKind {
    MenuItem(MenuItem),
    Submenu(Submenu),
    Predefined(PredefinedMenuItem),
    Check(CheckMenuItem),
    Icon(IconMenuItem),
}

impl Clone for MenuItemKind {
    fn clone(&self) -> Self {
        match self {
            Self::MenuItem(v) => Self::MenuItem(v.clone()),
            Self::Submenu(v) => Self::Submenu(v.clone()),
            Self::Predefined(v) => Self::Predefined(v.clone()),
            Self::Check(v) => Self::Check(v.clone()),
            Self::Icon(v) => Self::Icon(v.clone()),
        }
    }
}

impl MenuItemKind {
    // -------------------------------------------------------------------------
    // type
    // -------------------------------------------------------------------------

    pub fn as_menu_item(&self) -> Option<&MenuItem> {
        match self {
            Self::MenuItem(item) => Some(item),
            _ => None,
        }
    }

    pub fn as_submenu(&self) -> Option<&Submenu> {
        match self {
            Self::Submenu(item) => Some(item),
            _ => None,
        }
    }

    pub fn as_predefined(&self) -> Option<&PredefinedMenuItem> {
        match self {
            Self::Predefined(item) => Some(item),
            _ => None,
        }
    }

    pub fn as_check(&self) -> Option<&CheckMenuItem> {
        match self {
            Self::Check(item) => Some(item),
            _ => None,
        }
    }

    pub fn as_icon(&self) -> Option<&IconMenuItem> {
        match self {
            Self::Icon(item) => Some(item),
            _ => None,
        }
    }

    pub fn is_menu_item(&self) -> bool {
        matches!(self, Self::MenuItem(_))
    }

    pub fn is_submenu(&self) -> bool {
        matches!(self, Self::Submenu(_))
    }

    pub fn is_predefined(&self) -> bool {
        matches!(self, Self::Predefined(_))
    }

    pub fn is_check(&self) -> bool {
        matches!(self, Self::Check(_))
    }

    pub fn is_icon(&self) -> bool {
        matches!(self, Self::Icon(_))
    }

    // -------------------------------------------------------------------------
    // generic state
    // -------------------------------------------------------------------------

    pub fn text(&self) -> anyhow::Result<String> {
        match self {
            Self::MenuItem(item) => item.text(),
            Self::Submenu(item) => item.text(),
            Self::Predefined(item) => item.text(),
            Self::Check(item) => item.text(),
            Self::Icon(item) => item.text(),
        }
    }

    pub fn set_text<S: AsRef<str>>(&self, text: S) -> anyhow::Result<()> {
        match self {
            Self::MenuItem(item) => item.set_text(text),
            Self::Submenu(item) => item.set_text(text),
            Self::Predefined(item) => item.set_text(text),
            Self::Check(item) => item.set_text(text),
            Self::Icon(item) => item.set_text(text),
        }
    }

    pub fn is_enabled(&self) -> Option<anyhow::Result<bool>> {
        match self {
            Self::MenuItem(item) => Some(item.is_enabled()),
            Self::Submenu(item) => Some(item.is_enabled()),
            Self::Check(item) => Some(item.is_enabled()),
            Self::Icon(item) => Some(item.is_enabled()),
            Self::Predefined(_) => None,
        }
    }

    pub fn set_enabled(&self, enabled: bool) -> Option<anyhow::Result<()>> {
        match self {
            Self::MenuItem(item) => Some(item.set_enabled(enabled)),
            Self::Submenu(item) => Some(item.set_enabled(enabled)),
            Self::Check(item) => Some(item.set_enabled(enabled)),
            Self::Icon(item) => Some(item.set_enabled(enabled)),
            Self::Predefined(_) => None,
        }
    }

    pub fn is_checked(&self) -> Option<anyhow::Result<bool>> {
        match self {
            Self::Check(item) => Some(item.is_checked()),
            _ => None,
        }
    }

    pub fn set_checked(&self, checked: bool) -> Option<anyhow::Result<()>> {
        match self {
            Self::Check(item) => Some(item.set_checked(checked)),
            _ => None,
        }
    }

    // -------------------------------------------------------------------------
    // tree
    // -------------------------------------------------------------------------

    pub fn children(&self) -> anyhow::Result<Vec<MenuItemKind>> {
        match self {
            Self::Submenu(submenu) => submenu.items(),
            _ => Ok(Vec::new()),
        }
    }

    pub fn has_children(&self) -> bool {
        matches!(self, Self::Submenu(_))
    }

    pub fn find(&self, id: &MenuId) -> anyhow::Result<Option<MenuItemKind>> {
        if self.id() == id {
            return Ok(Some(self.clone()));
        }

        if let Self::Submenu(submenu) = self {
            for child in submenu.items()? {
                if let Some(found) = child.find(id)? {
                    return Ok(Some(found));
                }
            }
        }

        Ok(None)
    }

    pub fn descendants(&self) -> anyhow::Result<Vec<MenuItemKind>> {
        let mut result = Vec::new();
        self.collect_descendants(&mut result)?;
        Ok(result)
    }

    pub(crate) fn collect_descendants(&self, result: &mut Vec<MenuItemKind>) -> anyhow::Result<()> {
        if let Self::Submenu(submenu) = self {
            for child in submenu.items()? {
                result.push(child.clone());
                child.collect_descendants(result)?;
            }
        }

        Ok(())
    }

    pub fn id(&self) -> &MenuId {
        match self {
            Self::MenuItem(v) => v.id(),
            Self::Submenu(v) => v.id(),
            Self::Predefined(v) => v.id(),
            Self::Check(v) => v.id(),
            Self::Icon(v) => v.id(),
        }
    }

    pub(crate) fn inner_muda(&self) -> &dyn muda::IsMenuItem {
        match self {
            Self::MenuItem(v) => v.inner_muda(),
            Self::Submenu(v) => v.inner_muda(),
            Self::Predefined(v) => v.inner_muda(),
            Self::Check(v) => v.inner_muda(),
            Self::Icon(v) => v.inner_muda(),
        }
    }

    pub fn visit<F>(&self, visitor: &mut F) -> anyhow::Result<()>
    where
        F: FnMut(&MenuItemKind) -> anyhow::Result<()>,
    {
        visitor(self)?;

        if let Self::Submenu(submenu) = self {
            for child in submenu.items()? {
                child.visit(visitor)?;
            }
        }

        Ok(())
    }
}

/// A trait that defines a generic item in a menu, which may be one of [`MenuItemKind`]
///
/// # Safety
///
/// This trait is ONLY meant to be implemented internally by the crate.
pub trait IsMenuItem: sealed::IsMenuItemBase {
    /// Returns the kind of this menu item.
    fn kind(&self) -> MenuItemKind;

    /// Returns a unique identifier associated with this menu.
    fn id(&self) -> &MenuId;
}

macro_rules! impl_menu_item {
    ($ty:ident, $variant:ident) => {
        impl sealed::IsMenuItemBase for $ty {
            fn inner_muda(&self) -> &dyn muda::IsMenuItem {
                &self.0.inner
            }
        }

        impl IsMenuItem for $ty {
            fn kind(&self) -> MenuItemKind {
                MenuItemKind::$variant(self.clone())
            }

            fn id(&self) -> &MenuId {
                self.id()
            }
        }
    };
}

impl_menu_item!(MenuItem, MenuItem);
impl_menu_item!(Submenu, Submenu);
impl_menu_item!(PredefinedMenuItem, Predefined);
impl_menu_item!(CheckMenuItem, Check);
impl_menu_item!(IconMenuItem, Icon);

// -----------------------------------------------------------------------------
// normal item
// -----------------------------------------------------------------------------

impl MenuItem {
    pub fn new<T: AsRef<str>, A: AsRef<str>>(
        text: T,
        enabled: bool,
        accelerator: Option<A>,
    ) -> anyhow::Result<Self> {
        let accelerator = accelerator.and_then(|s| s.as_ref().parse().ok());
        let inner = muda::MenuItem::new(text.as_ref(), enabled, accelerator);
        Ok(Self(Arc::new(MenuItemInner::new(inner))))
    }

    pub fn with_id<I: Into<MenuId>, T: AsRef<str>, A: AsRef<str>>(
        id: I,
        text: T,
        enabled: bool,
        accelerator: Option<A>,
    ) -> anyhow::Result<Self> {
        let accelerator = accelerator.and_then(|s| s.as_ref().parse().ok());
        let inner = muda::MenuItem::with_id(id, text.as_ref(), enabled, accelerator);
        Ok(Self(Arc::new(MenuItemInner::new(inner))))
    }

    pub fn id(&self) -> &MenuId {
        self.0.inner.id()
    }

    pub fn text(&self) -> anyhow::Result<String> {
        Ok(self.0.inner.text())
    }

    pub fn set_text<S: AsRef<str>>(&self, text: S) -> anyhow::Result<()> {
        self.0.inner.set_text(text.as_ref());
        Ok(())
    }

    pub fn is_enabled(&self) -> anyhow::Result<bool> {
        Ok(self.0.inner.is_enabled())
    }

    pub fn set_enabled(&self, enabled: bool) -> anyhow::Result<()> {
        self.0.inner.set_enabled(enabled);
        Ok(())
    }

    pub fn set_accelerator<S: AsRef<str>>(&self, accelerator: Option<S>) -> anyhow::Result<()> {
        let accel = accelerator.and_then(|s| s.as_ref().parse().ok());
        self.0.inner.set_accelerator(accel).map_err(Into::into)
    }
}

// -----------------------------------------------------------------------------
// check item
// -----------------------------------------------------------------------------

impl CheckMenuItem {
    pub fn new<T: AsRef<str>, A: AsRef<str>>(
        text: T,
        enabled: bool,
        checked: bool,
        accelerator: Option<A>,
    ) -> anyhow::Result<Self> {
        let accelerator = accelerator.and_then(|s| s.as_ref().parse().ok());
        let inner = muda::CheckMenuItem::new(text.as_ref(), enabled, checked, accelerator);
        Ok(Self(Arc::new(CheckMenuItemInner::new(inner))))
    }

    pub fn with_id<I: Into<MenuId>, T: AsRef<str>, A: AsRef<str>>(
        id: I,
        text: T,
        enabled: bool,
        checked: bool,
        accelerator: Option<A>,
    ) -> anyhow::Result<Self> {
        let accelerator = accelerator.and_then(|s| s.as_ref().parse().ok());
        let inner = muda::CheckMenuItem::with_id(id, text.as_ref(), enabled, checked, accelerator);
        Ok(Self(Arc::new(CheckMenuItemInner::new(inner))))
    }

    pub fn id(&self) -> &MenuId {
        self.0.inner.id()
    }
    pub fn text(&self) -> anyhow::Result<String> {
        Ok(self.0.inner.text())
    }

    pub fn set_text<S: AsRef<str>>(&self, text: S) -> anyhow::Result<()> {
        self.0.inner.set_text(text.as_ref());
        Ok(())
    }

    pub fn is_enabled(&self) -> anyhow::Result<bool> {
        Ok(self.0.inner.is_enabled())
    }

    pub fn set_enabled(&self, enabled: bool) -> anyhow::Result<()> {
        self.0.inner.set_enabled(enabled);
        Ok(())
    }

    pub fn set_accelerator<S: AsRef<str>>(&self, accelerator: Option<S>) -> anyhow::Result<()> {
        let accel = accelerator.and_then(|s| s.as_ref().parse().ok());
        self.0.inner.set_accelerator(accel).map_err(Into::into)
    }

    pub fn is_checked(&self) -> anyhow::Result<bool> {
        Ok(self.0.inner.is_checked())
    }

    pub fn set_checked(&self, checked: bool) -> anyhow::Result<()> {
        self.0.inner.set_checked(checked);
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// icon item
// -----------------------------------------------------------------------------

impl IconMenuItem {
    pub fn new<T: AsRef<str>, A: AsRef<str>>(
        text: T,
        enabled: bool,
        icon: Option<Image<'_>>,
        accelerator: Option<A>,
    ) -> anyhow::Result<Self> {
        let accelerator = accelerator.and_then(|s| s.as_ref().parse().ok());
        let icon = icon.map(TryInto::try_into).transpose()?;
        let inner = muda::IconMenuItem::new(text.as_ref(), enabled, icon, accelerator);
        Ok(Self(Arc::new(IconMenuItemInner::new(inner))))
    }

    pub fn with_id<I: Into<MenuId>, T: AsRef<str>, A: AsRef<str>>(
        id: I,
        text: T,
        enabled: bool,
        icon: Option<Image<'_>>,
        accelerator: Option<A>,
    ) -> anyhow::Result<Self> {
        let accelerator = accelerator.and_then(|s| s.as_ref().parse().ok());
        let icon = icon.map(TryInto::try_into).transpose()?;
        let inner = muda::IconMenuItem::with_id(id, text.as_ref(), enabled, icon, accelerator);
        Ok(Self(Arc::new(IconMenuItemInner::new(inner))))
    }

    pub fn with_native_icon<T: AsRef<str>, A: AsRef<str>>(
        text: T,
        enabled: bool,
        icon: Option<NativeIcon>,
        accelerator: Option<A>,
    ) -> anyhow::Result<Self> {
        let accelerator = accelerator.and_then(|s| s.as_ref().parse().ok());
        let inner = muda::IconMenuItem::with_native_icon(
            text.as_ref(),
            enabled,
            icon.map(Into::into),
            accelerator,
        );
        Ok(Self(Arc::new(IconMenuItemInner::new(inner))))
    }

    pub fn with_id_and_native_icon<I: Into<MenuId>, T: AsRef<str>, A: AsRef<str>>(
        id: I,
        text: T,
        enabled: bool,
        icon: Option<NativeIcon>,
        accelerator: Option<A>,
    ) -> anyhow::Result<Self> {
        let accelerator = accelerator.and_then(|s| s.as_ref().parse().ok());
        let inner = muda::IconMenuItem::with_id_and_native_icon(
            id,
            text.as_ref(),
            enabled,
            icon.map(Into::into),
            accelerator,
        );
        Ok(Self(Arc::new(IconMenuItemInner::new(inner))))
    }

    pub fn id(&self) -> &MenuId {
        self.0.inner.id()
    }
    pub fn text(&self) -> anyhow::Result<String> {
        Ok(self.0.inner.text())
    }

    pub fn set_text<S: AsRef<str>>(&self, text: S) -> anyhow::Result<()> {
        self.0.inner.set_text(text.as_ref());
        Ok(())
    }

    pub fn is_enabled(&self) -> anyhow::Result<bool> {
        Ok(self.0.inner.is_enabled())
    }

    pub fn set_enabled(&self, enabled: bool) -> anyhow::Result<()> {
        self.0.inner.set_enabled(enabled);
        Ok(())
    }

    pub fn set_accelerator<S: AsRef<str>>(&self, accelerator: Option<S>) -> anyhow::Result<()> {
        let accel = accelerator.and_then(|s| s.as_ref().parse().ok());
        self.0.inner.set_accelerator(accel).map_err(Into::into)
    }

    pub fn set_icon(&self, icon: Option<Image<'_>>) -> anyhow::Result<()> {
        let icon = icon.map(TryInto::try_into).transpose()?;
        self.0.inner.set_icon(icon);
        Ok(())
    }

    pub fn set_native_icon(&self, icon: Option<NativeIcon>) -> anyhow::Result<()> {
        #[cfg(target_os = "macos")]
        self.0.inner.set_native_icon(icon.map(Into::into));
        let _ = icon;
        Ok(())
    }
}

impl MenuItemKind {
    pub(crate) fn from_muda(item: muda::MenuItemKind) -> Self {
        match item {
            muda::MenuItemKind::MenuItem(v) => {
                Self::MenuItem(MenuItem(Arc::new(MenuItemInner::new(v))))
            }
            muda::MenuItemKind::Submenu(v) => {
                Self::Submenu(Submenu(Arc::new(SubmenuInner::new(v))))
            }
            muda::MenuItemKind::Predefined(v) => Self::Predefined(PredefinedMenuItem(Arc::new(
                PredefinedMenuItemInner::new(v),
            ))),
            muda::MenuItemKind::Check(v) => {
                Self::Check(CheckMenuItem(Arc::new(CheckMenuItemInner::new(v))))
            }
            muda::MenuItemKind::Icon(v) => {
                Self::Icon(IconMenuItem(Arc::new(IconMenuItemInner::new(v))))
            }
        }
    }
}

// -----------------------------------------------------------------------------
// predefined items: direct muda calls, no manager and no main-thread dispatch
// -----------------------------------------------------------------------------

impl PredefinedMenuItem {
    fn wrap(inner: muda::PredefinedMenuItem) -> anyhow::Result<Self> {
        Ok(Self(Arc::new(PredefinedMenuItemInner::new(inner))))
    }

    pub fn separator() -> anyhow::Result<Self> {
        Self::wrap(muda::PredefinedMenuItem::separator())
    }
    pub fn copy(text: Option<&str>) -> anyhow::Result<Self> {
        Self::wrap(muda::PredefinedMenuItem::copy(text))
    }
    pub fn cut(text: Option<&str>) -> anyhow::Result<Self> {
        Self::wrap(muda::PredefinedMenuItem::cut(text))
    }
    pub fn paste(text: Option<&str>) -> anyhow::Result<Self> {
        Self::wrap(muda::PredefinedMenuItem::paste(text))
    }
    pub fn select_all(text: Option<&str>) -> anyhow::Result<Self> {
        Self::wrap(muda::PredefinedMenuItem::select_all(text))
    }
    pub fn undo(text: Option<&str>) -> anyhow::Result<Self> {
        Self::wrap(muda::PredefinedMenuItem::undo(text))
    }
    pub fn redo(text: Option<&str>) -> anyhow::Result<Self> {
        Self::wrap(muda::PredefinedMenuItem::redo(text))
    }
    pub fn minimize(text: Option<&str>) -> anyhow::Result<Self> {
        Self::wrap(muda::PredefinedMenuItem::minimize(text))
    }
    pub fn maximize(text: Option<&str>) -> anyhow::Result<Self> {
        Self::wrap(muda::PredefinedMenuItem::maximize(text))
    }
    pub fn fullscreen(text: Option<&str>) -> anyhow::Result<Self> {
        Self::wrap(muda::PredefinedMenuItem::fullscreen(text))
    }
    pub fn hide(text: Option<&str>) -> anyhow::Result<Self> {
        Self::wrap(muda::PredefinedMenuItem::hide(text))
    }
    pub fn hide_others(text: Option<&str>) -> anyhow::Result<Self> {
        Self::wrap(muda::PredefinedMenuItem::hide_others(text))
    }
    pub fn show_all(text: Option<&str>) -> anyhow::Result<Self> {
        Self::wrap(muda::PredefinedMenuItem::show_all(text))
    }
    pub fn close_window(text: Option<&str>) -> anyhow::Result<Self> {
        Self::wrap(muda::PredefinedMenuItem::close_window(text))
    }
    pub fn quit(text: Option<&str>) -> anyhow::Result<Self> {
        Self::wrap(muda::PredefinedMenuItem::quit(text))
    }
    pub fn services(text: Option<&str>) -> anyhow::Result<Self> {
        Self::wrap(muda::PredefinedMenuItem::services(text))
    }
    pub fn bring_all_to_front(text: Option<&str>) -> anyhow::Result<Self> {
        Self::wrap(muda::PredefinedMenuItem::bring_all_to_front(text))
    }

    pub fn about(text: Option<&str>, metadata: Option<AboutMetadata<'_>>) -> anyhow::Result<Self> {
        let metadata = metadata.map(TryInto::try_into).transpose()?;
        Self::wrap(muda::PredefinedMenuItem::about(text, metadata))
    }

    pub fn id(&self) -> &MenuId {
        self.0.inner.id()
    }
    pub fn text(&self) -> anyhow::Result<String> {
        Ok(self.0.inner.text())
    }

    pub fn set_text<S: AsRef<str>>(&self, text: S) -> anyhow::Result<()> {
        self.0.inner.set_text(text.as_ref());
        Ok(())
    }
}

impl sealed::IsMenuItemBase for MenuItemKind {
    fn inner_muda(&self) -> &dyn muda::IsMenuItem {
        self.inner_muda()
    }
}

impl IsMenuItem for MenuItemKind {
    fn kind(&self) -> MenuItemKind {
        self.clone()
    }

    fn id(&self) -> &MenuId {
        self.id()
    }
}

impl Menu {
    /// Direct children only.
    pub fn get(&self, id: &MenuId) -> Result<Option<MenuItemKind>> {
        Ok(self.items()?.into_iter().find(|item| item.id() == id))
    }
    pub fn visit<F>(&self, mut visitor: F) -> Result<()>
    where
        F: FnMut(&MenuItemKind) -> Result<()>,
    {
        for item in self.items()? {
            item.visit(&mut visitor)?;
        }
        Ok(())
    }
    /// Search the entire menu tree recursively.
    pub fn find(&self, id: &MenuId) -> Result<Option<MenuItemKind>> {
        for item in self.items()? {
            if let Some(found) = item.find(id)? {
                return Ok(Some(found));
            }
        }
        Ok(None)
    }
    /// Every item in the tree, depth-first.
    pub fn all_items(&self) -> Result<Vec<MenuItemKind>> {
        let mut result = Vec::new();
        for item in self.items()? {
            result.push(item.clone());
            item.collect_descendants(&mut result)?;
        }
        Ok(result)
    }
    pub fn get_menu_item(&self, id: &MenuId) -> Result<Option<MenuItem>> {
        Ok(match self.find(id)? {
            Some(MenuItemKind::MenuItem(item)) => Some(item),
            _ => None,
        })
    }
    pub fn get_submenu(&self, id: &MenuId) -> Result<Option<Submenu>> {
        Ok(match self.find(id)? {
            Some(MenuItemKind::Submenu(item)) => Some(item),
            _ => None,
        })
    }
    pub fn get_check(&self, id: &MenuId) -> Result<Option<CheckMenuItem>> {
        Ok(match self.find(id)? {
            Some(MenuItemKind::Check(item)) => Some(item),
            _ => None,
        })
    }
    pub fn get_icon(&self, id: &MenuId) -> Result<Option<IconMenuItem>> {
        Ok(match self.find(id)? {
            Some(MenuItemKind::Icon(item)) => Some(item),
            _ => None,
        })
    }
    pub fn get_predefined(&self, id: &MenuId) -> Result<Option<PredefinedMenuItem>> {
        Ok(match self.find(id)? {
            Some(MenuItemKind::Predefined(item)) => Some(item),
            _ => None,
        })
    }
}
impl Submenu {
    /// Direct child only.
    pub fn get(&self, id: &MenuId) -> Result<Option<MenuItemKind>> {
        Ok(self.items()?.into_iter().find(|item| item.id() == id))
    }
    /// Recursive search starting from this submenu.
    pub fn find(&self, id: &MenuId) -> Result<Option<MenuItemKind>> {
        for item in self.items()? {
            if let Some(found) = item.find(id)? {
                return Ok(Some(found));
            }
        }
        Ok(None)
    }
    pub fn visit<F>(&self, mut visitor: F) -> Result<()>
    where
        F: FnMut(&MenuItemKind) -> Result<()>,
    {
        for item in self.items()? {
            item.visit(&mut visitor)?;
        }
        Ok(())
    }
    pub fn all_items(&self) -> Result<Vec<MenuItemKind>> {
        let mut result = Vec::new();
        for item in self.items()? {
            result.push(item.clone());
            item.collect_descendants(&mut result)?;
        }
        Ok(result)
    }
    pub fn get_menu_item(&self, id: &MenuId) -> Result<Option<MenuItem>> {
        Ok(match self.find(id)? {
            Some(MenuItemKind::MenuItem(item)) => Some(item),
            _ => None,
        })
    }
    pub fn get_submenu(&self, id: &MenuId) -> Result<Option<Submenu>> {
        Ok(match self.find(id)? {
            Some(MenuItemKind::Submenu(item)) => Some(item),
            _ => None,
        })
    }
    pub fn get_check(&self, id: &MenuId) -> Result<Option<CheckMenuItem>> {
        Ok(match self.find(id)? {
            Some(MenuItemKind::Check(item)) => Some(item),
            _ => None,
        })
    }
    pub fn get_icon(&self, id: &MenuId) -> Result<Option<IconMenuItem>> {
        Ok(match self.find(id)? {
            Some(MenuItemKind::Icon(item)) => Some(item),
            _ => None,
        })
    }
    pub fn get_predefined(&self, id: &MenuId) -> Result<Option<PredefinedMenuItem>> {
        Ok(match self.find(id)? {
            Some(MenuItemKind::Predefined(item)) => Some(item),
            _ => None,
        })
    }
}
#[cfg(target_os = "macos")]
impl Submenu {
    pub fn set_as_windows_menu_for_nsapp(&self) {
        self.0.inner.set_as_windows_menu_for_nsapp();
    }
    pub fn set_as_help_menu_for_nsapp(&self) {
        self.0.inner.set_as_help_menu_for_nsapp();
    }
}
impl Menu {
    pub fn inner_muda(&self) -> &muda::Menu {
        &self.0.inner
    }
}
impl Submenu {
    pub fn inner_muda_ref(&self) -> &muda::Submenu {
        &self.0.inner
    }
    pub fn inner(&self) -> muda::Submenu {
        self.0.inner.clone()
    }
}
impl Submenu {
    pub fn new<S: AsRef<str>>(text: S, enabled: bool) -> Result<Self> {
        Ok(Self(Arc::new(SubmenuInner::new(muda::Submenu::new(
            text.as_ref(),
            enabled,
        )))))
    }
    pub fn with_id<I: Into<MenuId>, S: AsRef<str>>(id: I, text: S, enabled: bool) -> Result<Self> {
        Ok(Self(Arc::new(SubmenuInner::new(muda::Submenu::with_id(
            id,
            text.as_ref(),
            enabled,
        )))))
    }
    pub fn new_with_icon<S: AsRef<str>>(
        text: S,
        enabled: bool,
        icon: Option<Image<'_>>,
    ) -> Result<Self> {
        let submenu = muda::Submenu::new(text.as_ref(), enabled);
        if let Some(icon) = icon {
            submenu.set_icon(Some(icon.try_into()?));
        }
        Ok(Self(Arc::new(SubmenuInner::new(submenu))))
    }
    pub fn with_id_and_icon<I: Into<MenuId>, S: AsRef<str>>(
        id: I,
        text: S,
        enabled: bool,
        icon: Option<Image<'_>>,
    ) -> Result<Self> {
        let submenu = muda::Submenu::with_id(id, text.as_ref(), enabled);
        if let Some(icon) = icon {
            submenu.set_icon(Some(icon.try_into()?));
        }
        Ok(Self(Arc::new(SubmenuInner::new(submenu))))
    }
    pub fn new_with_native_icon<S: AsRef<str>>(
        text: S,
        enabled: bool,
        icon: Option<NativeIcon>,
    ) -> Result<Self> {
        let submenu = muda::Submenu::new(text.as_ref(), enabled);
        if let Some(icon) = icon {
            submenu.set_native_icon(Some(icon.into()));
        }
        Ok(Self(Arc::new(SubmenuInner::new(submenu))))
    }
    pub fn with_id_and_native_icon<I: Into<MenuId>, S: AsRef<str>>(
        id: I,
        text: S,
        enabled: bool,
        icon: Option<NativeIcon>,
    ) -> Result<Self> {
        let submenu = muda::Submenu::with_id(id, text.as_ref(), enabled);
        if let Some(icon) = icon {
            submenu.set_native_icon(Some(icon.into()));
        }
        Ok(Self(Arc::new(SubmenuInner::new(submenu))))
    }
    pub fn with_items<S: AsRef<str>>(
        text: S,
        enabled: bool,
        items: &[&dyn IsMenuItem],
    ) -> Result<Self> {
        let submenu = Self::new(text, enabled)?;
        submenu.append_items(items)?;
        Ok(submenu)
    }
    pub fn with_id_and_items<I: Into<MenuId>, S: AsRef<str>>(
        id: I,
        text: S,
        enabled: bool,
        items: &[&dyn IsMenuItem],
    ) -> Result<Self> {
        let submenu = Self::with_id(id, text, enabled)?;
        submenu.append_items(items)?;
        Ok(submenu)
    }
    pub fn id(&self) -> &MenuId {
        self.0.inner.id()
    }
    pub fn text(&self) -> Result<String> {
        Ok(self.0.inner.text())
    }
    pub fn append(&self, item: &dyn IsMenuItem) -> Result<()> {
        self.0.inner.append(item.inner_muda()).map_err(Into::into)
    }
    pub fn append_items(&self, items: &[&dyn IsMenuItem]) -> Result<()> {
        for item in items {
            self.append(*item)?;
        }
        Ok(())
    }
    pub fn prepend(&self, item: &dyn IsMenuItem) -> Result<()> {
        self.0.inner.prepend(item.inner_muda()).map_err(Into::into)
    }
    pub fn insert(&self, item: &dyn IsMenuItem, position: usize) -> Result<()> {
        self.0
            .inner
            .insert(item.inner_muda(), position)
            .map_err(Into::into)
    }
    pub fn remove(&self, item: &dyn IsMenuItem) -> Result<()> {
        self.0.inner.remove(item.inner_muda()).map_err(Into::into)
    }
    pub fn items(&self) -> Result<Vec<MenuItemKind>> {
        Ok(self
            .0
            .inner
            .items()
            .into_iter()
            .map(MenuItemKind::from_muda)
            .collect())
    }
    pub fn set_text<S: AsRef<str>>(&self, text: S) -> Result<()> {
        self.0.inner.set_text(text.as_ref());
        Ok(())
    }
    pub fn is_enabled(&self) -> Result<bool> {
        Ok(self.0.inner.is_enabled())
    }
    pub fn set_enabled(&self, enabled: bool) -> Result<()> {
        self.0.inner.set_enabled(enabled);
        Ok(())
    }
    pub fn set_icon(&self, icon: Option<Image<'_>>) -> Result<()> {
        let icon = icon.map(TryInto::try_into).transpose()?;
        self.0.inner.set_icon(icon);
        Ok(())
    }
    pub fn set_native_icon(&self, icon: Option<NativeIcon>) -> Result<()> {
        #[cfg(target_os = "macos")]
        self.0.inner.set_native_icon(icon.map(Into::into));
        let _ = icon;
        Ok(())
    }
}
// -----------------------------------------------------------------------------
// menu
// -----------------------------------------------------------------------------
impl Menu {
    pub fn new() -> Result<Self> {
        Ok(Self(Arc::new(MenuInner::new(muda::Menu::new()))))
    }
    pub fn with_id<I: Into<MenuId>>(id: I) -> Result<Self> {
        Ok(Self(Arc::new(MenuInner::new(muda::Menu::with_id(id)))))
    }
    pub fn with_items(items: &[&dyn IsMenuItem]) -> Result<Self> {
        let menu = Self::new()?;
        menu.append_items(items)?;
        Ok(menu)
    }
    pub fn with_id_and_items<I: Into<MenuId>>(id: I, items: &[&dyn IsMenuItem]) -> Result<Self> {
        let menu = Self::with_id(id)?;
        menu.append_items(items)?;
        Ok(menu)
    }
    pub fn id(&self) -> &MenuId {
        self.0.inner.id()
    }
    pub fn append(&self, item: &dyn IsMenuItem) -> Result<()> {
        self.0.inner.append(item.inner_muda()).map_err(Into::into)
    }
    pub fn append_items(&self, items: &[&dyn IsMenuItem]) -> Result<()> {
        for item in items {
            self.append(*item)?;
        }
        Ok(())
    }
    pub fn prepend(&self, item: &dyn IsMenuItem) -> Result<()> {
        self.0.inner.prepend(item.inner_muda()).map_err(Into::into)
    }
    pub fn insert(&self, item: &dyn IsMenuItem, position: usize) -> Result<()> {
        self.0
            .inner
            .insert(item.inner_muda(), position)
            .map_err(Into::into)
    }
    pub fn remove(&self, item: &dyn IsMenuItem) -> Result<()> {
        self.0.inner.remove(item.inner_muda()).map_err(Into::into)
    }
    pub fn items(&self) -> Result<Vec<MenuItemKind>> {
        Ok(self
            .0
            .inner
            .items()
            .into_iter()
            .map(MenuItemKind::from_muda)
            .collect())
    }
}
impl Menu {
    // -------------------------------------------------------------------------
    // Windows
    // -------------------------------------------------------------------------
    #[cfg(windows)]
    /// Initializes this menu for the specified Windows window handle.
    ///
    /// # Safety
    ///
    /// `hwnd` must be a valid Windows window handle accepted by the underlying
    /// `muda` implementation. The caller must ensure that the handle remains
    /// valid for as long as the operation requires it.
    pub unsafe fn init_for_hwnd(&self, hwnd: isize) -> Result<()> {
        unsafe { self.0.inner.init_for_hwnd(hwnd).map_err(Into::into) }
    }
    #[cfg(windows)]
    /// Initializes this menu for the specified Windows window handle and theme.
    ///
    /// # Safety
    ///
    /// `hwnd` must be a valid Windows window handle accepted by the underlying
    /// `muda` implementation. The caller must ensure that the handle remains
    /// valid for as long as the operation requires it.
    pub unsafe fn init_for_hwnd_with_theme(
        &self,
        hwnd: isize,
        theme: muda::MenuTheme,
    ) -> Result<()> {
        unsafe {
            self.0
                .inner
                .init_for_hwnd_with_theme(hwnd, theme)
                .map_err(Into::into)
        }
    }
    #[cfg(windows)]
    /// Sets the menu theme for the specified Windows window handle.
    ///
    /// # Safety
    ///
    /// `hwnd` must be a valid Windows window handle accepted by the underlying
    /// `muda` implementation. The caller must ensure that the handle is still
    /// valid when the theme is changed.
    pub unsafe fn set_theme_for_hwnd(&self, hwnd: isize, theme: muda::MenuTheme) -> Result<()> {
        unsafe {
            self.0
                .inner
                .set_theme_for_hwnd(hwnd, theme)
                .map_err(Into::into)
        }
    }
    #[cfg(windows)]
    pub fn haccel(&self) -> isize {
        self.0.inner.haccel()
    }
    pub fn inner(&self) -> muda::Menu {
        self.0.inner.clone()
    }
    // -------------------------------------------------------------------------
    // macOS
    // -------------------------------------------------------------------------
    #[cfg(target_os = "macos")]
    pub fn init_for_nsapp(&self) {
        self.0.inner.init_for_nsapp();
    }
    // -------------------------------------------------------------------------
    // Linux / BSD — GTK
    // -------------------------------------------------------------------------
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    pub fn init_for_gtk_window<W, C>(&self, window: &W, container: Option<&C>) -> Result<()>
    where
        W: gtk::prelude::IsA<gtk::Window> + gtk::prelude::IsA<gtk::Widget>,
        C: gtk::prelude::IsA<gtk::Widget>,
    {
        self.0
            .inner
            .init_for_gtk_window(window, container)
            .map_err(Into::into)
    }
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    pub fn remove_for_gtk_window<W>(&self, window: &W) -> Result<()>
    where
        W: gtk::prelude::IsA<gtk::Window> + gtk::prelude::IsA<gtk::Widget>,
    {
        self.0
            .inner
            .remove_for_gtk_window(window)
            .map_err(Into::into)
    }
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    pub fn hide_for_gtk_window<W>(&self, window: &W) -> Result<()>
    where
        W: gtk::prelude::IsA<gtk::Window>,
    {
        self.0.inner.hide_for_gtk_window(window).map_err(Into::into)
    }
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    pub fn show_for_gtk_window<W>(&self, window: &W) -> Result<()>
    where
        W: gtk::prelude::IsA<gtk::Window>,
    {
        self.0.inner.show_for_gtk_window(window).map_err(Into::into)
    }
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    pub fn is_visible_on_gtk_window<W>(&self, window: &W) -> bool
    where
        W: gtk::prelude::IsA<gtk::Window>,
    {
        self.0.inner.is_visible_on_gtk_window(window)
    }
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    pub fn gtk_menubar_for_gtk_window<W>(self, window: &W) -> Option<gtk::MenuBar>
    where
        W: gtk::prelude::IsA<gtk::Window>,
    {
        self.0.inner.clone().gtk_menubar_for_gtk_window(window)
    }
}

/// Application metadata for the [`PredefinedMenuItem::about`].
#[derive(Debug, Clone, Default)]
pub struct AboutMetadata<'a> {
    /// Sets the application name.
    pub name: Option<String>,
    /// The application version.
    pub version: Option<String>,
    /// The short version, e.g. "1.0".
    ///
    /// ## Platform-specific
    ///
    /// - **Windows / Linux:** Appended to the end of `version` in parentheses.
    pub short_version: Option<String>,
    /// The authors of the application.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Unsupported.
    pub authors: Option<Vec<String>>,
    /// Application comments.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Unsupported.
    pub comments: Option<String>,
    /// The copyright of the application.
    pub copyright: Option<String>,
    /// The license of the application.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Unsupported.
    pub license: Option<String>,
    /// The application website.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Unsupported.
    pub website: Option<String>,
    /// The website label.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Unsupported.
    pub website_label: Option<String>,
    /// The credits.
    ///
    /// ## Platform-specific
    ///
    /// - **Windows / Linux:** Unsupported.
    pub credits: Option<String>,
    /// The application icon.
    ///
    /// ## Platform-specific
    ///
    /// - **Windows:** Unsupported.
    pub icon: Option<Image<'a>>,
}

/// A builder type for [`AboutMetadata`].
#[derive(Clone, Debug, Default)]
pub struct AboutMetadataBuilder<'a>(AboutMetadata<'a>);

impl<'a> AboutMetadataBuilder<'a> {
    /// Create a new about metadata builder.
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the application name.
    pub fn name<S: Into<String>>(mut self, name: Option<S>) -> Self {
        self.0.name = name.map(|s| s.into());
        self
    }
    /// Sets the application version.
    pub fn version<S: Into<String>>(mut self, version: Option<S>) -> Self {
        self.0.version = version.map(|s| s.into());
        self
    }
    /// Sets the short version, e.g. "1.0".
    ///
    /// ## Platform-specific
    ///
    /// - **Windows / Linux:** Appended to the end of `version` in parentheses.
    pub fn short_version<S: Into<String>>(mut self, short_version: Option<S>) -> Self {
        self.0.short_version = short_version.map(|s| s.into());
        self
    }
    /// Sets the authors of the application.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Unsupported.
    pub fn authors(mut self, authors: Option<Vec<String>>) -> Self {
        self.0.authors = authors;
        self
    }
    /// Application comments.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Unsupported.
    pub fn comments<S: Into<String>>(mut self, comments: Option<S>) -> Self {
        self.0.comments = comments.map(|s| s.into());
        self
    }
    /// Sets the copyright of the application.
    pub fn copyright<S: Into<String>>(mut self, copyright: Option<S>) -> Self {
        self.0.copyright = copyright.map(|s| s.into());
        self
    }
    /// Sets the license of the application.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Unsupported.
    pub fn license<S: Into<String>>(mut self, license: Option<S>) -> Self {
        self.0.license = license.map(|s| s.into());
        self
    }
    /// Sets the application website.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Unsupported.
    pub fn website<S: Into<String>>(mut self, website: Option<S>) -> Self {
        self.0.website = website.map(|s| s.into());
        self
    }
    /// Sets the website label.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Unsupported.
    pub fn website_label<S: Into<String>>(mut self, website_label: Option<S>) -> Self {
        self.0.website_label = website_label.map(|s| s.into());
        self
    }
    /// Sets the credits.
    ///
    /// ## Platform-specific
    ///
    /// - **Windows / Linux:** Unsupported.
    pub fn credits<S: Into<String>>(mut self, credits: Option<S>) -> Self {
        self.0.credits = credits.map(|s| s.into());
        self
    }
    /// Sets the application icon.
    ///
    /// ## Platform-specific
    ///
    /// - **Windows:** Unsupported.
    pub fn icon(mut self, icon: Option<Image<'a>>) -> Self {
        self.0.icon = icon;
        self
    }

    /// Construct the final [`AboutMetadata`]
    pub fn build(self) -> AboutMetadata<'a> {
        self.0
    }
}

impl TryFrom<AboutMetadata<'_>> for muda::AboutMetadata {
    type Error = anyhow::Error;

    fn try_from(value: AboutMetadata<'_>) -> Result<Self, Self::Error> {
        let icon = match value.icon {
            Some(i) => Some(i.try_into()?),
            None => None,
        };

        Ok(Self {
            authors: value.authors,
            name: value.name,
            version: value.version,
            short_version: value.short_version,
            comments: value.comments,
            copyright: value.copyright,
            license: value.license,
            website: value.website,
            website_label: value.website_label,
            credits: value.credits,
            icon,
        })
    }
}

/// A native Icon to be used for the menu item
///
/// ## Platform-specific:
///
/// - **Windows / Linux**: Unsupported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum NativeIcon {
    /// An add item template image.
    Add,
    /// Advanced preferences toolbar icon for the preferences window.
    Advanced,
    /// A Bluetooth template image.
    Bluetooth,
    /// Bookmarks image suitable for a template.
    Bookmarks,
    /// A caution image.
    Caution,
    /// A color panel toolbar icon.
    ColorPanel,
    /// A column view mode template image.
    ColumnView,
    /// A computer icon.
    Computer,
    /// An enter full-screen mode template image.
    EnterFullScreen,
    /// Permissions for all users.
    Everyone,
    /// An exit full-screen mode template image.
    ExitFullScreen,
    /// A cover flow view mode template image.
    FlowView,
    /// A folder image.
    Folder,
    /// A burnable folder icon.
    FolderBurnable,
    /// A smart folder icon.
    FolderSmart,
    /// A link template image.
    FollowLinkFreestanding,
    /// A font panel toolbar icon.
    FontPanel,
    /// A `go back` template image.
    GoLeft,
    /// A `go forward` template image.
    GoRight,
    /// Home image suitable for a template.
    Home,
    /// An iChat Theater template image.
    IChatTheater,
    /// An icon view mode template image.
    IconView,
    /// An information toolbar icon.
    Info,
    /// A template image used to denote invalid data.
    InvalidDataFreestanding,
    /// A generic left-facing triangle template image.
    LeftFacingTriangle,
    /// A list view mode template image.
    ListView,
    /// A locked padlock template image.
    LockLocked,
    /// An unlocked padlock template image.
    LockUnlocked,
    /// A horizontal dash, for use in menus.
    MenuMixedState,
    /// A check mark template image, for use in menus.
    MenuOnState,
    /// A MobileMe icon.
    MobileMe,
    /// A drag image for multiple items.
    MultipleDocuments,
    /// A network icon.
    Network,
    /// A path button template image.
    Path,
    /// General preferences toolbar icon for the preferences window.
    PreferencesGeneral,
    /// A Quick Look template image.
    QuickLook,
    /// A refresh template image.
    RefreshFreestanding,
    /// A refresh template image.
    Refresh,
    /// A remove item template image.
    Remove,
    /// A reveal contents template image.
    RevealFreestanding,
    /// A generic right-facing triangle template image.
    RightFacingTriangle,
    /// A share view template image.
    Share,
    /// A slideshow template image.
    Slideshow,
    /// A badge for a `smart` item.
    SmartBadge,
    /// Small green indicator, similar to iChat's available image.
    StatusAvailable,
    /// Small clear indicator.
    StatusNone,
    /// Small yellow indicator, similar to iChat's idle image.
    StatusPartiallyAvailable,
    /// Small red indicator, similar to iChat's unavailable image.
    StatusUnavailable,
    /// A stop progress template image.
    StopProgressFreestanding,
    /// A stop progress button template image.
    StopProgress,
    /// An image of the empty trash can.
    TrashEmpty,
    /// An image of the full trash can.
    TrashFull,
    /// Permissions for a single user.
    User,
    /// User account toolbar icon for the preferences window.
    UserAccounts,
    /// Permissions for a group of users.
    UserGroup,
    /// Permissions for guests.
    UserGuest,
}

impl From<NativeIcon> for muda::NativeIcon {
    fn from(value: NativeIcon) -> Self {
        match value {
            NativeIcon::Add => muda::NativeIcon::Add,
            NativeIcon::Advanced => muda::NativeIcon::Advanced,
            NativeIcon::Bluetooth => muda::NativeIcon::Bluetooth,
            NativeIcon::Bookmarks => muda::NativeIcon::Bookmarks,
            NativeIcon::Caution => muda::NativeIcon::Caution,
            NativeIcon::ColorPanel => muda::NativeIcon::ColorPanel,
            NativeIcon::ColumnView => muda::NativeIcon::ColumnView,
            NativeIcon::Computer => muda::NativeIcon::Computer,
            NativeIcon::EnterFullScreen => muda::NativeIcon::EnterFullScreen,
            NativeIcon::Everyone => muda::NativeIcon::Everyone,
            NativeIcon::ExitFullScreen => muda::NativeIcon::ExitFullScreen,
            NativeIcon::FlowView => muda::NativeIcon::FlowView,
            NativeIcon::Folder => muda::NativeIcon::Folder,
            NativeIcon::FolderBurnable => muda::NativeIcon::FolderBurnable,
            NativeIcon::FolderSmart => muda::NativeIcon::FolderSmart,
            NativeIcon::FollowLinkFreestanding => muda::NativeIcon::FollowLinkFreestanding,
            NativeIcon::FontPanel => muda::NativeIcon::FontPanel,
            NativeIcon::GoLeft => muda::NativeIcon::GoLeft,
            NativeIcon::GoRight => muda::NativeIcon::GoRight,
            NativeIcon::Home => muda::NativeIcon::Home,
            NativeIcon::IChatTheater => muda::NativeIcon::IChatTheater,
            NativeIcon::IconView => muda::NativeIcon::IconView,
            NativeIcon::Info => muda::NativeIcon::Info,
            NativeIcon::InvalidDataFreestanding => muda::NativeIcon::InvalidDataFreestanding,
            NativeIcon::LeftFacingTriangle => muda::NativeIcon::LeftFacingTriangle,
            NativeIcon::ListView => muda::NativeIcon::ListView,
            NativeIcon::LockLocked => muda::NativeIcon::LockLocked,
            NativeIcon::LockUnlocked => muda::NativeIcon::LockUnlocked,
            NativeIcon::MenuMixedState => muda::NativeIcon::MenuMixedState,
            NativeIcon::MenuOnState => muda::NativeIcon::MenuOnState,
            NativeIcon::MobileMe => muda::NativeIcon::MobileMe,
            NativeIcon::MultipleDocuments => muda::NativeIcon::MultipleDocuments,
            NativeIcon::Network => muda::NativeIcon::Network,
            NativeIcon::Path => muda::NativeIcon::Path,
            NativeIcon::PreferencesGeneral => muda::NativeIcon::PreferencesGeneral,
            NativeIcon::QuickLook => muda::NativeIcon::QuickLook,
            NativeIcon::RefreshFreestanding => muda::NativeIcon::RefreshFreestanding,
            NativeIcon::Refresh => muda::NativeIcon::Refresh,
            NativeIcon::Remove => muda::NativeIcon::Remove,
            NativeIcon::RevealFreestanding => muda::NativeIcon::RevealFreestanding,
            NativeIcon::RightFacingTriangle => muda::NativeIcon::RightFacingTriangle,
            NativeIcon::Share => muda::NativeIcon::Share,
            NativeIcon::Slideshow => muda::NativeIcon::Slideshow,
            NativeIcon::SmartBadge => muda::NativeIcon::SmartBadge,
            NativeIcon::StatusAvailable => muda::NativeIcon::StatusAvailable,
            NativeIcon::StatusNone => muda::NativeIcon::StatusNone,
            NativeIcon::StatusPartiallyAvailable => muda::NativeIcon::StatusPartiallyAvailable,
            NativeIcon::StatusUnavailable => muda::NativeIcon::StatusUnavailable,
            NativeIcon::StopProgressFreestanding => muda::NativeIcon::StopProgressFreestanding,
            NativeIcon::StopProgress => muda::NativeIcon::StopProgress,
            NativeIcon::TrashEmpty => muda::NativeIcon::TrashEmpty,
            NativeIcon::TrashFull => muda::NativeIcon::TrashFull,
            NativeIcon::User => muda::NativeIcon::User,
            NativeIcon::UserAccounts => muda::NativeIcon::UserAccounts,
            NativeIcon::UserGroup => muda::NativeIcon::UserGroup,
            NativeIcon::UserGuest => muda::NativeIcon::UserGuest,
        }
    }
}

// -----------------------------------------------------------------------------
// RawWindow
// -----------------------------------------------------------------------------

/// Platform-specific view of a freshly created native window.
///
/// The lifetime is tied directly to the underlying Tao window. This prevents a
/// `RawWindow` from outliving the native window from which its platform handles
/// were obtained.
///
/// A `RawWindow` is intentionally short-lived. It is created for callbacks and
/// native initialization and must not be stored beyond those operations.
pub struct RawWindow<'a> {
    #[cfg(windows)]
    pub hwnd: isize,
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    pub gtk_window: &'a gtk::ApplicationWindow,
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    pub default_vbox: Option<&'a gtk::Box>,
    pub _marker: &'a std::marker::PhantomData<()>,
}

pub struct WindowMenu {
    pub is_app_wide: bool,
    pub menu: Menu,
}
