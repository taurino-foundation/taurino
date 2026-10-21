use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use {
    pyo3::{exceptions::PyRuntimeError, prelude::*},
    std::sync::OnceLock,
};

use ipckit::SharedMemory;
#[cfg(windows)]
use tao::platform::windows::EventLoopBuilderExtWindows;
use {
    anyhow::{Context, Result},
    tao::{
        event::{Event, WindowEvent},
        event_loop::ControlFlow,
        window::WindowBuilder,
    },
    taurino_native::events::{EventLoop, EventLoopBuilder, EventLoopProxy},
    wry::WebViewBuilder,
};

use crate::{
    managers::{Manager, menu::MenuManager},
    utils::spawn_shutdown_watcher,
};

pub mod managers;
pub mod utils;
pub struct Taurino {
    runner: EventLoop,
    manager: Arc<Manager>,
    proxy: EventLoopProxy,
}

impl Taurino {
    pub fn new(config: Vec<u8>) -> Result<Self> {
        let _ = config;
        // Initialize native menu management.
        let menu_manager = MenuManager::new()?;

        // Create the native event loop.
        let mut builder = EventLoopBuilder::with_user_event();

        // Install the Windows native message hook.
        #[cfg(windows)]
        {
            builder.with_msg_hook(MenuManager::install_msg_hook(menu_manager.clone()));
        }

        let runner = builder.build();
        let proxy = runner.create_proxy();

        let manager = Manager::new(menu_manager)?;

        Ok(Self {
            runner,
            manager,
            proxy,
        })
    }

    pub fn run(
        self,
        shutdown_event: Py<PyAny>,
        notify_ready: Py<PyAny>,
        shared_memory_name: &str,
    ) -> Result<()> {
        let Self {
            runner,
            manager,
            proxy,
        } = self;
        let watcher_running = Arc::new(AtomicBool::new(true));
        let watcher =
            spawn_shutdown_watcher(shutdown_event, proxy.clone(), Arc::clone(&watcher_running));

        // Notify Python only once the IPC client, window, and WebView exist.
        // This does NOT mean JavaScript has installed all application listeners.
        if let Err(error) = Python::attach(|py| notify_ready.call0(py).map(|_| ())) {
            watcher_running.store(false, Ordering::Release);
            let _ = watcher.join();
            return Err(error.into());
        }

        // Open the region created by the Python parent.
        let _shared_memory = SharedMemory::open(shared_memory_name)
            .context("Failed to open Taurino shared memory")?;

        // Create the initial native window.
        let window = WindowBuilder::new()
            .with_title("Taurino")
            .with_inner_size(dpi::LogicalSize::new(1000.0, 700.0))
            .build(&runner)?;
        let _webview = WebViewBuilder::new()
            .with_url("https://www.y-sic.com")
            .build(&window)?;
        let window_id = window.id();

        // Start the native application event loop.
        runner.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    window_id: id,
                    event: WindowEvent::CloseRequested,
                    ..
                } if id == window_id => {
                    *control_flow = ControlFlow::Exit;
                }

                _ => {}
            }

            // Keep application resources alive for the lifetime
            // of the native event loop.
            let _ = (&window, &manager, &proxy);
        })
    }

    pub fn manager(&self) -> &Arc<Manager> {
        &self.manager
    }

    pub fn proxy(&self) -> &EventLoopProxy {
        &self.proxy
    }
}

pub fn get_taurino_version() -> &'static str {
    static VERSION: OnceLock<String> = OnceLock::new();

    VERSION.get_or_init(|| {
        let version = env!("CARGO_PKG_VERSION");
        version.replace("-alpha", "a").replace("-beta", "b")
    })
}

fn run_runtime(
    config: Vec<u8>,
    shutdown_event: Py<PyAny>,
    notify_ready: Py<PyAny>,
    shared_memory_name: &str,
) -> Result<()> {
    // Initialize Taurino.
    let app = Taurino::new(config).context("Failed to initialize Taurino Runtime")?;

    // Start the native GUI with the shared memory handle.
    app.run(shutdown_event, notify_ready, shared_memory_name)
        .context("Taurino Runtime failed")
}

#[pyfunction]
pub fn runtime(
    py: Python<'_>,
    config: Vec<u8>,
    shutdown_event: Py<PyAny>,
    notify_ready: Py<PyAny>,
    shared_memory_name: String,
) -> PyResult<()> {
    py.detach(move || {
        run_runtime(config, shutdown_event, notify_ready, &shared_memory_name)
            .map_err(|error| PyRuntimeError::new_err(format!("{error:#}")))
    })
}

#[pymodule]
fn _taurino(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("__version__", get_taurino_version())?;
    module.add_function(wrap_pyfunction!(runtime, module)?)?;
    Ok(())
}
