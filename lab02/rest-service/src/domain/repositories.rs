use crate::domain::image::StoredImage;
use crate::domain::product::Product;
use crate::error::AppResult;

pub trait ProductRepository: Send + Sync {
    fn create(&self, name: String, description: String) -> AppResult<Product>;
    fn get(&self, id: u64) -> AppResult<Product>;
    fn update(&self, product: Product) -> AppResult<Product>;
    fn delete(&self, id: u64) -> AppResult<Product>;
    fn list(&self) -> AppResult<Vec<Product>>;
}

pub trait ImageStorage: Send + Sync {
    fn save(&self, name: String, bytes: Vec<u8>) -> AppResult<StoredImage>;

    fn load(&self, key: String) -> AppResult<StoredImage>;
}
