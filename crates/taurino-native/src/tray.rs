use crate::{config::Rect, image::Image, menu::ContextMenu, resources::Resource};
use std::{path::Path, sync::Arc};
pub use tray_icon::TrayIconId;

#[derive(Default)]
pub struct TrayIconBuilder {
    inner: tray_icon::TrayIconBuilder,
}

impl TrayIconBuilder {
    pub fn new() -> Self {
        Self {
            inner: tray_icon::TrayIconBuilder::new(),
        }
    }

    pub fn with_id<I: Into<TrayIconId>>(id: I) -> Self {
        let mut builder = Self::new();
        builder.inner = builder.inner.with_id(id);
        builder
    }

    pub fn menu<M: ContextMenu>(mut self, menu: &M) -> Self {
        self.inner = self.inner.with_menu(menu.inner_context_owned());
        self
    }

    pub fn icon(mut self, icon: Image<'_>) -> Self {
        if let Ok(icon) = icon.try_into() {
            self.inner = self.inner.with_icon(icon);
        }
        self
    }

    pub fn tooltip<S: AsRef<str>>(mut self, tooltip: S) -> Self {
        self.inner = self.inner.with_tooltip(tooltip);
        self
    }

    pub fn title<S: AsRef<str>>(mut self, title: S) -> Self {
        self.inner = self.inner.with_title(title);
        self
    }

    pub fn temp_dir_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.inner = self.inner.with_temp_dir_path(path);
        self
    }

    pub fn icon_as_template(mut self, is_template: bool) -> Self {
        self.inner = self.inner.with_icon_as_template(is_template);
        self
    }

    #[deprecated(
        since = "2.2.0",
        note = "Use `TrayIconBuilder::show_menu_on_left_click` instead."
    )]
    pub fn menu_on_left_click(mut self, enable: bool) -> Self {
        self.inner = self.inner.with_menu_on_left_click(enable);
        self
    }

    pub fn show_menu_on_left_click(mut self, enable: bool) -> Self {
        self.inner = self.inner.with_menu_on_left_click(enable);
        self
    }

    pub fn id(&self) -> &TrayIconId {
        self.inner.id()
    }

    /// Build directly. No manager/app handle argument is required.
    pub fn build(self) -> anyhow::Result<TrayIcon> {
        let Self { inner } = self;
        let tray = TrayIcon::from_inner(inner.build()?);
        Ok(tray)
    }
}
pub struct TrayIconInner {
    pub id: TrayIconId,
    pub inner: tray_icon::TrayIcon,
}

/// Standalone tray icon wrapper.
///
/// This type is reference-counted. The native icon is removed when the last
/// `TrayIcon` clone is dropped.
#[derive(Clone)]
pub struct TrayIcon {
    pub inner: Arc<TrayIconInner>,
}

impl TrayIcon {
    pub fn from_inner(inner: tray_icon::TrayIcon) -> Self {
        /* install_global_dispatch(); */
        let id = inner.id().clone();
        Self {
            inner: Arc::new(TrayIconInner { id, inner }),
        }
    }

    pub fn id(&self) -> &TrayIconId {
        &self.inner.id
    }

    pub fn set_icon(&self, icon: Option<Image<'_>>) -> anyhow::Result<()> {
        let icon = icon.map(TryInto::try_into).transpose()?;
        self.inner.inner.set_icon(icon).map_err(Into::into)
    }

    pub fn set_menu<M: ContextMenu + 'static>(&self, menu: Option<M>) -> anyhow::Result<()> {
        self.inner
            .inner
            .set_menu(menu.map(|m| m.inner_context_owned()));
        Ok(())
    }

    pub fn set_tooltip<S: AsRef<str>>(&self, tooltip: Option<S>) -> anyhow::Result<()> {
        self.inner
            .inner
            .set_tooltip(tooltip.map(|s| s.as_ref().to_string()))
            .map_err(Into::into)
    }

    pub fn set_title<S: AsRef<str>>(&self, title: Option<S>) -> anyhow::Result<()> {
        self.inner
            .inner
            .set_title(title.map(|s| s.as_ref().to_string()));
        Ok(())
    }

    pub fn set_visible(&self, visible: bool) -> anyhow::Result<()> {
        self.inner.inner.set_visible(visible).map_err(Into::into)
    }

    pub fn set_temp_dir_path<P: AsRef<Path>>(&self, path: Option<P>) -> anyhow::Result<()> {
        #[cfg(target_os = "linux")]
        self.inner
            .inner
            .set_temp_dir_path(path.map(|p| p.as_ref().to_path_buf()));
        let _ = path;
        Ok(())
    }

    pub fn set_icon_as_template(&self, #[allow(unused)] is_template: bool) -> anyhow::Result<()> {
        #[cfg(target_os = "macos")]
        self.inner.inner.set_icon_as_template(is_template);
        Ok(())
    }

    pub fn set_icon_with_as_template(
        &self,
        icon: Option<Image<'_>>,
        #[allow(unused)] is_template: bool,
    ) -> anyhow::Result<()> {
        #[cfg(target_os = "macos")]
        {
            let icon = icon.map(TryInto::try_into).transpose()?;

            self.inner
                .inner
                .set_icon_with_as_template(icon, is_template)?;

            Ok(())
        }

        #[cfg(not(target_os = "macos"))]
        {
            self.set_icon(icon)
        }
    }

    pub fn set_show_menu_on_left_click(&self, #[allow(unused)] enable: bool) -> anyhow::Result<()> {
        #[cfg(any(target_os = "macos", windows))]
        self.inner.inner.set_show_menu_on_left_click(enable);
        Ok(())
    }

    pub fn rect(&self) -> anyhow::Result<Option<Rect>> {
        Ok(self.inner.inner.rect().map(|rect| Rect {
            position: rect.position.into(),
            size: rect.size.into(),
        }))
    }

    pub fn with_inner_tray_icon<F, T>(&self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(&tray_icon::TrayIcon) -> T,
    {
        Ok(f(&self.inner.inner))
    }
}

impl Resource for TrayIcon {}

// Same invariant as the standalone menu wrappers: creation and native mutation
// are expected to happen on the GUI/main thread.
unsafe impl Send for TrayIconInner {}
unsafe impl Sync for TrayIconInner {}
