#[derive(Debug, Clone)]
pub struct Config {
    pub output_dir: std::path::PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_dir: std::path::PathBuf::from("./output"),
        }
    }
}
