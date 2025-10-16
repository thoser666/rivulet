use anyhow::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;

// ==============
// TRAIT DEFINITIONEN
// ==============

/// Ein Trait, um `Box<dyn Output>` klonbar zu machen.
pub trait DynClone {
    fn clone_box(&self) -> Box<dyn Output>;
}

impl<T> DynClone for T
where
    T: 'static + Output + Clone,
{
    fn clone_box(&self) -> Box<dyn Output> {
        Box::new(self.clone())
    }
}

// `Output` erfordert jetzt `Send`, `Sync`, `Debug` und `DynClone`.
#[async_trait::async_trait]
pub trait Output: Send + Sync + Debug + DynClone {
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
}

// ==============
// STRUKTUR DEFINITIONEN
// ==============

#[derive(Debug, Clone, Default)]
pub struct Scene {
    // Platzhalter für Szenen-Eigenschaften
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    // Platzhalter für Konfigurations-Eigenschaften
}

#[derive(Debug, Default)]
pub struct RivuletEngine {
    scenes: Arc<RwLock<HashMap<Uuid, Scene>>>,
    active_scene: Arc<RwLock<Option<Uuid>>>,
    outputs: Arc<RwLock<Vec<Box<dyn Output>>>>,
    // config: Arc<RwLock<Config>>, // Auskommentiert, bis es verwendet wird
}

// ==============
// IMPLEMENTIERUNGEN
// ==============

impl RivuletEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_scene(&self) -> Uuid {
        let mut scenes = self.scenes.write();
        let id = Uuid::new_v4();
        scenes.insert(id, Scene::default());
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

    pub fn get_active_scene_id(&self) -> Option<Uuid> {
        *self.active_scene.read()
    }

    pub async fn start_all_outputs(&self) -> Result<()> {
        // Klone die Boxen, um die Sperre schnell wieder freizugeben
        let outputs_to_start = self
            .outputs
            .read()
            .iter()
            .map(|o| o.clone_box())
            .collect::<Vec<_>>();

        for output in outputs_to_start {
            output.start().await?;
        }
        Ok(())
    }

    pub async fn stop_all_outputs(&self) -> Result<()> {
        let outputs_to_stop = self
            .outputs
            .read()
            .iter()
            .map(|o| o.clone_box())
            .collect::<Vec<_>>();

        for output in outputs_to_stop {
            output.stop().await?;
        }
        Ok(())
    }

    pub fn add_output(&self, output: Box<dyn Output>) {
        self.outputs.write().push(output);
    }
}
