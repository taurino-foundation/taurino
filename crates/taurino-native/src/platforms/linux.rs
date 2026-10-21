use crate::{
    config::PhysicalRect,
    platforms::{MonitorExt, WindowExt},
};
use dpi::{LogicalPosition, LogicalSize};
use gtk::{gdk::prelude::MonitorExt as GdkMonitorExt, prelude::WidgetExt};
use tao::platform::unix::{MonitorHandleExtUnix, WindowExtUnix};

impl MonitorExt for tao::monitor::MonitorHandle {
    fn work_area(&self) -> PhysicalRect<i32, u32> {
        let rect = self.gdk_monitor().workarea();
        let scale_factor = self.scale_factor();

        PhysicalRect {
            size: LogicalSize::new(rect.width() as u32, rect.height() as u32)
                .to_physical(scale_factor),

            position: LogicalPosition::new(rect.x(), rect.y()).to_physical(scale_factor),
        }
    }
}

impl WindowExt for tao::window::Window {
    fn set_enabled(&self, enabled: bool) {
        self.gtk_window().set_sensitive(enabled);
    }

    fn is_enabled(&self) -> bool {
        self.gtk_window().is_sensitive()
    }

    fn center(&self) {
        if let Some(monitor) = self.current_monitor() {
            let window_size = self.outer_size();

            let new_pos = crate::utils::calculate_window_center_position(window_size, monitor);

            self.set_outer_position(new_pos);
        }
    }
}
