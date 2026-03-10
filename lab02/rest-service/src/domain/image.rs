use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult};

#[derive(Clone, Debug)]
pub struct StoredImage {
    file_path: PathBuf,
    size_bytes: usize,
    bytes: Vec<u8>,
    content_type: String,
}

impl StoredImage {
    pub fn new(file_path: PathBuf, size_bytes: usize, bytes: Vec<u8>) -> AppResult<Self> {
        if file_path.as_os_str().is_empty() {
            return Err(AppError::BadRequest("Filepath cannot be empty".into()));
        }

        let content_type = content_type_for_path(&file_path)
            .ok_or_else(|| AppError::BadRequest("Unsupported image type".into()))?;

        Ok(Self {
            file_path,
            size_bytes,
            bytes,
            content_type: content_type.to_string(),
        })
    }

    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    pub fn into_file_path(self) -> PathBuf {
        self.file_path
    }

    pub fn size_bytes(&self) -> usize {
        self.size_bytes
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    pub fn content_type(&self) -> &str {
        &self.content_type
    }
}

pub fn content_type(content_type: &str) -> Option<&'static str> {
    let normalized = content_type.split(';').next()?.trim();
    match normalized {
        "image/png" => Some("png"),
        "image/jpeg" => Some("jpg"),
        "image/gif" => Some("gif"),
        "image/webp" => Some("webp"),
        _ => None,
    }
}

pub fn content_type_for_path(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}
