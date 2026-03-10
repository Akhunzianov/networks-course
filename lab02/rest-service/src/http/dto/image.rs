use serde::{Deserialize, Serialize};

use crate::domain::image::StoredImage;

#[derive(Deserialize)]
pub struct AttachIconRequest {
    pub bytes: Vec<u8>,
}

#[derive(Serialize)]
pub struct IconResponse {
    pub bytes: Vec<u8>,
}

impl From<StoredImage> for IconResponse {
    fn from(image: StoredImage) -> Self {
        Self {
            bytes: image.into_bytes(),
        }
    }
}

impl From<&StoredImage> for IconResponse {
    fn from(image: &StoredImage) -> Self {
        Self {
            bytes: image.bytes().to_vec(),
        }
    }
}
