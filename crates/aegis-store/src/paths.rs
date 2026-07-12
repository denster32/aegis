use directories::ProjectDirs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AegisPaths {
    pub root: PathBuf,
    pub db: PathBuf,
    pub artifacts: PathBuf,
    pub config: PathBuf,
}

impl AegisPaths {
    pub fn default_dirs() -> anyhow::Result<Self> {
        let proj = ProjectDirs::from("dev", "aegis", "aegis")
            .ok_or_else(|| anyhow::anyhow!("cannot resolve config dirs"))?;
        let root = proj.data_dir().to_path_buf();
        Ok(Self::from_root(root))
    }

    pub fn from_root(root: PathBuf) -> Self {
        Self {
            db: root.join("aegis.db"),
            artifacts: root.join("artifacts"),
            config: root.join("config.toml"),
            root,
        }
    }

    pub fn ensure(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.root)?;
        std::fs::create_dir_all(&self.artifacts)?;
        Ok(())
    }
}
