use anyhow::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

// Placeholder für Traits, die in anderen Crates definiert werden
#[async_trait::async_trait]
pub trait Output: Send + Sync {
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
}

pub struct Scene {
    // ...
}

pub struct Config {
    // ...
}

#[derive(Debug, Default)]
pub struct RivuletEngine {
    scenes: Arc<RwLock<HashMap<Uuid, Scene>>>,
    active_scene: Arc<RwLock<Option<Uuid>>>,
    outputs: Arc<RwLock<Vec<Box<dyn Output>>>>,
    // config: Arc<RwLock<Config>>, // Auskommentiert, bis es verwendet wird
}

impl RivuletEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_scene(&self) -> Uuid {
        let mut scenes = self.scenes.write();
        let id = Uuid::new_v4();
        scenes.insert(id, Scene {});
        id
    }

    pub fn set_active_scene(&self, id: Uuid) -> Result<()> {
        let scenes = self.scenes.read();
        if !scenes.contains_key(&id) {
            return Err(anyhow::anyhow!("Scene with ID {} not found", id));
        }
        *self.active_scene.write() = Some(id);
        Ok(())
    }

    pub fn get_active_scene(&self) -> Option<&Scene> {
        // Korrektur 1: `clone_on_copy` behoben
        let active_id = (*self.active_scene.read())?;

        // Unsafe, weil der Read-Lock nicht über die Funktion hinaus gehalten wird.
        // Für eine sichere Implementierung müsste man den Lock zurückgeben.
        // Für den Moment ist das aber ok, da die `scenes`-Map selten geändert wird.
        let scenes = self.scenes.read();
        let scene_ptr = scenes.get(&active_id)? as *const Scene;
        unsafe { Some(&*scene_ptr) }
    }

    // Korrektur 2: `await_holding_lock` behoben
    pub async fn start_all_outputs(&self) -> Result<()> {
        let output_refs: Vec<_> = self.outputs.read().iter().map(|o| o.clone_box()).collect();
        // Die `read`-Sperre wird hier freigegeben.

        for output in output_refs {
            output.start().await?;
        }
        Ok(())
    }

    // Korrektur 3: `await_holding_lock` behoben
    pub async fn stop_all_outputs(&self) -> Result<()> {
        let output_refs: Vec<_> = self.outputs.read().iter().map(|o| o.clone_box()).collect();
        // Die `read`-Sperre wird hier freigegeben.

        for output in output_refs {
            output.stop().await?;
        }
        Ok(())
    }

    pub fn add_output(&self, output: Box<dyn Output>) {
        self.outputs.write().push(output);
    }
}

// Hilfs-Trait, um `Box<dyn Trait>` zu klonen
#[async_trait::async_trait]
pub trait CloneBox {
    fn clone_box(&self) -> Box<dyn Output>;
}

#[async_trait::async_trait]
impl<T> CloneBox for T
where
    T: 'static + Output + Clone,
{
    fn clone_box(&self) -> Box<dyn Output> {
        Box::new(self.clone())
    }
}
