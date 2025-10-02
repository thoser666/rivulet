use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Scene {
    pub id: Uuid,
    pub name: String,
}

impl Scene {
    pub fn new(name: String) -> Self {
        Self { id: Uuid::new_v4(), name }
    }
}
