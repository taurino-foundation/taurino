use std::sync::Arc;

use {anyhow::Result, parking_lot::Mutex};

use crate::managers::Manager;

pub struct WindowManager {
    manager: Option<Arc<Manager>>,
}

impl WindowManager {
    pub fn new() -> Result<Arc<Mutex<Self>>> {
        Ok(Arc::new(Mutex::new(Self { manager: None })))
    }

    pub(crate) fn bind_manager(&mut self, manager: Arc<Manager>) {
        self.manager = Some(manager)
    }
}
