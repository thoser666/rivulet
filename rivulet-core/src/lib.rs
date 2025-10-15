use parking_lot::RwLock;
use std::sync::Arc;
use uuid::Uuid;

pub mod config;
pub mod output;
pub mod scene;
pub mod source;

pub use config::*;
pub use output::*;
pub use scene::*;
pub use source::*;

/// The main Rivulet engine that manages scenes, sources, and outputs
#[derive(Debug)]
pub struct RivuletEngine {
    scenes: Arc<RwLock<Vec<Scene>>>,
    active_scene: Arc<RwLock<Option<Uuid>>>,
    outputs: Arc<RwLock<Vec<Box<dyn Output>>>>,
    config: Arc<RwLock<Config>>,
}

impl RivuletEngine {
    pub fn new() -> anyhow::Result<Self> {
        tracing::info!("Initializing Rivulet Engine");

        Ok(Self {
            scenes: Arc::new(RwLock::new(Vec::new())),
            active_scene: Arc::new(RwLock::new(None)),
            outputs: Arc::new(RwLock::new(Vec::new())),
            config: Arc::new(RwLock::new(Config::default())),
        })
    }

    pub fn add_scene(&self, scene: Scene) -> anyhow::Result<Uuid> {
        let scene_id = scene.id;
        let mut scenes = self.scenes.write();
        scenes.push(scene);

        if scenes.len() == 1 {
            *self.active_scene.write() = Some(scene_id);
        }

        tracing::info!("Added scene: {}", scene_id);
        Ok(scene_id)
    }

    pub fn switch_scene(&self, scene_id: Uuid) -> anyhow::Result<()> {
        let scenes = self.scenes.read();
        if scenes.iter().any(|s| s.id == scene_id) {
            *self.active_scene.write() = Some(scene_id);
            tracing::info!("Switched to scene: {}", scene_id);
            Ok(())
        } else {
            anyhow::bail!("Scene not found: {}", scene_id);
        }
    }

    pub fn get_active_scene(&self) -> Option<Scene> {
        let active_id = self.active_scene.read().clone()?;
        let scenes = self.scenes.read();
        scenes.iter().find(|s| s.id == active_id).cloned()
    }

    pub fn add_output(&self, output: Box<dyn Output>) -> anyhow::Result<()> {
        let mut outputs = self.outputs.write();
        outputs.push(output);
        tracing::info!("Added output, total: {}", outputs.len());
        Ok(())
    }

    pub async fn start_output(&self) -> anyhow::Result<()> {
        let outputs = self.outputs.read();
        for output in outputs.iter() {
            output.start().await?;
        }
        tracing::info!("Started all outputs");
        Ok(())
    }

    pub async fn stop_output(&self) -> anyhow::Result<()> {
        let outputs = self.outputs.read();
        for output in outputs.iter() {
            output.stop().await?;
        }
        tracing::info!("Stopped all outputs");
        Ok(())
    }
}
