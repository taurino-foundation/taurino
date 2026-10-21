use {crate::config::PhysicalRect, dpi::LogicalSize};

use {
    objc2::MainThreadMarker,
    objc2_app_kit::{NSBackingStoreType, NSWindow, NSWindowStyleMask},
    tao::platform::macos::WindowExtMacOS,
};

impl crate::platforms::WindowExt for tao::window::Window {
    // based on electron implementation
    // https://github.com/electron/electron/blob/15db63e26df3e3d59ce6281f030624f746518511/shell/browser/native_window_mac.mm#L474
    fn set_enabled(&self, enabled: bool) {
        let ns_window: &NSWindow = unsafe { &*self.ns_window().cast() };
        if !enabled {
            let frame = ns_window.frame();
            let mtm = MainThreadMarker::new()
                .expect("`Window::set_enabled` can only be called on the main thread");
            let sheet = unsafe {
                NSWindow::initWithContentRect_styleMask_backing_defer(
                    mtm.alloc(),
                    frame,
                    NSWindowStyleMask::Titled,
                    NSBackingStoreType::Buffered,
                    false,
                )
            };
            sheet.setAlphaValue(0.5);
            ns_window.beginSheet_completionHandler(&sheet, None);
        } else if let Some(attached) = ns_window.attachedSheet() {
            ns_window.endSheet(&attached);
        }
    }

    fn is_enabled(&self) -> bool {
        let ns_window: &NSWindow = unsafe { &*self.ns_window().cast() };
        ns_window.attachedSheet().is_none()
    }

    fn center(&self) {
        let ns_window: &NSWindow = unsafe { &*self.ns_window().cast() };
        ns_window.center();
    }
}

impl crate::platforms::MonitorExt for tao::monitor::MonitorHandle {
    fn work_area(&self) -> PhysicalRect<i32, u32> {
        use {objc2_app_kit::NSScreen, tao::platform::macos::MonitorHandleExtMacOS};
        if let Some(ns_screen) = self.ns_screen() {
            let ns_screen: &NSScreen = unsafe { &*ns_screen.cast() };
            let screen_frame = ns_screen.frame();
            let visible_frame = ns_screen.visibleFrame();

            let scale_factor = self.scale_factor();

            let mut position = self.position().to_logical::<f64>(scale_factor);

            position.x += visible_frame.origin.x - screen_frame.origin.x;

            position.y += (screen_frame.origin.y + screen_frame.size.height)
                - (visible_frame.origin.y + visible_frame.size.height);
            PhysicalRect {
                size: LogicalSize::new(visible_frame.size.width, visible_frame.size.height)
                    .to_physical(scale_factor),
                position: position.to_physical(scale_factor),
            }
        } else {
            PhysicalRect {
                size: self.size(),
                position: self.position(),
            }
        }
    }
}
