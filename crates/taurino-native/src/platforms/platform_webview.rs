#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
))]
mod imp {
    pub type Webview = webkit2gtk::WebView;
}

#[cfg(target_vendor = "apple")]
mod imp {
    use std::ffi::c_void;

    // Borrowed Objective-C pointers owned elsewhere.
    // These pointers must not be mutated through.
    pub struct Webview {
        pub webview: *mut c_void,
        pub manager: *mut c_void,

        #[cfg(target_os = "macos")]
        pub ns_window: *mut c_void,

        #[cfg(target_os = "ios")]
        pub view_controller: *mut c_void,
    }
}

#[cfg(windows)]
mod imp {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        ICoreWebView2Controller, ICoreWebView2Environment,
    };

    pub struct Webview {
        pub controller: ICoreWebView2Controller,
        pub environment: ICoreWebView2Environment,
    }
}

#[cfg(target_os = "android")]
mod imp {
    pub type Webview = wry::JniHandle;
}

use imp::Webview;

/// A platform-specific WebView handle.
///
/// This type provides access to native WebView objects through
/// platform-specific methods.
///
/// The underlying native handles remain owned by the WebView
/// implementation.
pub struct PlatformWebview(Webview);

impl PlatformWebview {
    /// Returns the native WebKitGTK WebView handle.
    ///
    /// # Platform
    ///
    /// Linux, DragonFly BSD, FreeBSD, NetBSD and OpenBSD.
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
        )))
    )]
    pub fn inner(&self) -> webkit2gtk::WebView {
        self.0.clone()
    }

    /// Returns the native WebView2 controller.
    ///
    /// # Platform
    ///
    /// Windows.
    #[cfg(windows)]
    #[cfg_attr(docsrs, doc(cfg(windows)))]
    pub fn controller(
        &self,
    ) -> webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller {
        self.0.controller.clone()
    }

    /// Returns the native WebView2 environment.
    ///
    /// # Platform
    ///
    /// Windows.
    #[cfg(windows)]
    #[cfg_attr(docsrs, doc(cfg(windows)))]
    pub fn environment(
        &self,
    ) -> webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Environment {
        self.0.environment.clone()
    }

    /// Returns the native WKWebView handle.
    ///
    /// # Platform
    ///
    /// macOS and iOS.
    ///
    /// # Safety
    ///
    /// The returned pointer is borrowed. The caller must not
    /// release it or use it after the owning WebView is destroyed.
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    #[cfg_attr(docsrs, doc(cfg(any(target_os = "macos", target_os = "ios"))))]
    pub fn inner(&self) -> *mut std::ffi::c_void {
        self.0.webview
    }

    /// Returns the native WKUserContentController handle.
    ///
    /// # Platform
    ///
    /// macOS and iOS.
    ///
    /// The pointer is borrowed from the owning WebView.
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    #[cfg_attr(docsrs, doc(cfg(any(target_os = "macos", target_os = "ios"))))]
    pub fn controller(&self) -> *mut std::ffi::c_void {
        self.0.manager
    }

    /// Returns the NSWindow associated with the WebView.
    ///
    /// # Platform
    ///
    /// macOS.
    ///
    /// The pointer is borrowed from the owning native window.
    #[cfg(target_os = "macos")]
    #[cfg_attr(docsrs, doc(cfg(target_os = "macos")))]
    pub fn ns_window(&self) -> *mut std::ffi::c_void {
        self.0.ns_window
    }

    /// Returns the UIViewController associated with the WebView.
    ///
    /// # Platform
    ///
    /// iOS.
    ///
    /// The pointer is borrowed from the owning native controller.
    #[cfg(target_os = "ios")]
    #[cfg_attr(docsrs, doc(cfg(target_os = "ios")))]
    pub fn view_controller(&self) -> *mut std::ffi::c_void {
        self.0.view_controller
    }

    /// Returns the native JNI handle.
    ///
    /// # Platform
    ///
    /// Android.
    #[cfg(target_os = "android")]
    #[cfg_attr(docsrs, doc(cfg(target_os = "android")))]
    pub fn jni_handle(&self) -> wry::JniHandle {
        self.0
    }
}
