use std::sync::Arc;

use tracing::{info, warn};

use crate::{
    domain::image::{StoredImage, content_type},
    domain::product::Product,
    domain::repositories::ImageStorage,
    domain::repositories::ProductRepository,
    error::{AppError, AppResult},
};

#[derive(Clone, Debug)]
pub struct CreateProductInput {
    pub name: String,
    pub description: String,
}

#[derive(Clone, Debug)]
pub struct UpdateProductInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AttachIconInput {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

#[derive(Clone)]
pub struct ProductService {
    repo: Arc<dyn ProductRepository>,
    image_repo: Arc<dyn ImageStorage>,
}

impl ProductService {
    pub fn new(repo: Arc<dyn ProductRepository>, image_repo: Arc<dyn ImageStorage>) -> Self {
        Self { repo, image_repo }
    }

    pub async fn create_product(&self, input: CreateProductInput) -> AppResult<Product> {
        info!(
            name = %input.name,
            description_len = input.description.len(),
            "service create_product"
        );
        self.repo.create(input.name, input.description)
    }

    pub async fn get_product(&self, id: u64) -> AppResult<Product> {
        info!(product_id = id, "service get_product");
        self.repo.get(id)
    }

    pub async fn update_product(&self, id: u64, input: UpdateProductInput) -> AppResult<Product> {
        info!(
            product_id = id,
            name_present = input.name.is_some(),
            description_present = input.description.is_some(),
            "service update_product"
        );
        let mut product = self.repo.get(id)?;
        product.update(input.name, input.description, None)?;
        self.repo.update(product)
    }

    pub async fn delete_product(&self, id: u64) -> AppResult<Product> {
        info!(product_id = id, "service delete_product");
        self.repo.delete(id)
    }

    pub async fn list_products(&self) -> AppResult<Vec<Product>> {
        info!("service list_products");
        self.repo.list()
    }

    pub async fn attach_icon(&self, id: u64, input: AttachIconInput) -> AppResult<StoredImage> {
        let cont_type = content_type(&input.content_type)
            .ok_or_else(|| AppError::BadRequest("Unsupported image content type".into()))?;
        let file_name = format!("{}_product_icon.{}", id, cont_type);
        info!(
            product_id = id,
            file_name = %file_name,
            content_type = %input.content_type,
            bytes_len = input.bytes.len(),
            "service attach_icon"
        );
        let stored_image = self.image_repo.save(file_name, input.bytes)?;
        let mut product = self.repo.get(id)?;
        product.update(None, None, Some(stored_image.file_path().to_path_buf()))?;
        self.repo.update(product)?;
        Ok(stored_image)
    }

    pub async fn get_attached_icon(&self, id: u64) -> AppResult<StoredImage> {
        info!(product_id = id, "service get_attached_icon");
        let product = self.repo.get(id)?;
        let icon_str = match product.icon() {
            Some(icon) => icon,
            None => {
                warn!(product_id = id, "service get_attached_icon missing icon");
                return Err(AppError::BadRequest("No image have been loaded".into()));
            }
        };
        self.image_repo.load(icon_str.to_string())
    }
}
