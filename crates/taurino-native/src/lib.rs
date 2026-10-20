pub mod config;
pub mod events;
pub mod image;
pub mod menu;
pub mod platforms;
pub mod resources;
pub mod tray;
pub mod utils;
pub mod webview;
pub mod window;
mod wrappers;

// ============================================================
// Windows Low-Level Reexports
// ============================================================

#[cfg(windows)]
pub use {once_cell, softbuffer, webview2_com};

#[cfg(windows)]
pub use ::windows as windows_api;
// ============================================================
// macOS Low-Level Reexports
// ============================================================

#[cfg(target_os = "macos")]
pub use {
    block2, objc2, objc2_app_kit, objc2_foundation, objc2_web_kit, security_framework, x_win,
    x509_parser,
};

// ============================================================
// Linux / BSD Low-Level Reexports
// ============================================================

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
))]
pub use {gtk, webkit2gtk};
