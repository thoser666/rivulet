pub mod scene;
pub mod source;
pub mod output;
pub mod config;

pub use scene::*;
pub use source::*;
pub use output::*;
pub use config::*;

use std::sync::Arc;
use parking_lot::RwLock;

pub struct RivuletEngine {
    scenes: Arc<RwLock<Vec<Scene>>>,
}

impl RivuletEngine {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            scenes: Arc::new(RwLock::new(Vec::new())),
        })
    }
}
