pub mod api;
pub mod types;
pub mod plugin;

pub struct PluginManager;

impl PluginManager {
    pub fn initialize() -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct PluginSystem;

impl PluginSystem {
    pub fn new() -> Self { Self }
    pub fn discover_obs_plugins(&mut self) -> anyhow::Result<Vec<()>> { Ok(vec![]) }
    pub fn auto_load_plugins(&mut self) -> anyhow::Result<Vec<String>> { Ok(vec![]) }
}
