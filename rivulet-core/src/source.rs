use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Source {
    pub id: Uuid,
    pub name: String,
}

impl Source {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
        }
    }
}
