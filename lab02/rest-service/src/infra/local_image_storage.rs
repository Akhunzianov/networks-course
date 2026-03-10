use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use tracing::{error, info};

use crate::{
    domain::image::StoredImage,
    domain::repositories::ImageStorage,
    error::{AppError, AppResult},
};

pub struct LocalImageStorage {
    root_dir: Mutex<PathBuf>,
}

impl LocalImageStorage {
    pub fn new() -> AppResult<Self> {
        let dir = env::current_dir().map_err(|_| AppError::Internal)?;
        let root_dir = dir.join("assets");
        fs::create_dir_all(&root_dir).map_err(|err| {
            error!(error = %err, "failed to create assets directory");
            AppError::Internal
        })?;
        info!(root_dir = %root_dir.display(), "local image storage ready");
        Ok(Self {
            root_dir: Mutex::new(root_dir),
        })
    }

    fn lock_root_dir(&self) -> AppResult<MutexGuard<'_, PathBuf>> {
        self.root_dir.lock().map_err(|_| {
            error!("image storage mutex poisoned");
            AppError::Internal
        })
    }
}

impl ImageStorage for LocalImageStorage {
    fn save(&self, name: String, bytes: Vec<u8>) -> AppResult<StoredImage> {
        let root = self.lock_root_dir()?;
        let mut path = root.clone();
        path.push(&name);
        info!(
            file_name = %name,
            bytes_len = bytes.len(),
            "image storage save"
        );
        fs::write(&path, &bytes).map_err(|err| {
            error!(error = %err, file_path = %path.display(), "failed to save image");
            AppError::Internal
        })?;
        StoredImage::new(path, bytes.len(), bytes)
    }

    fn load(&self, name: String) -> AppResult<StoredImage> {
        let root = self.lock_root_dir()?;
        let mut path = root.clone();
        path.push(&name);
        info!(file_name = %name, "image storage load");
        let bytes = fs::read(&path).map_err(|err| {
            error!(error = %err, file_path = %path.display(), "failed to load image");
            AppError::Internal
        })?;
        StoredImage::new(path, bytes.len(), bytes)
    }
}
