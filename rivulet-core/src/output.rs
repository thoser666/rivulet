use async_trait::async_trait;

#[async_trait]
pub trait Output: Send + Sync {
    async fn start(&self) -> anyhow::Result<()>;
    async fn stop(&self) -> anyhow::Result<()>;
}
