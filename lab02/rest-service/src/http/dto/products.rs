use serde::{Deserialize, Serialize};

use crate::domain::product::Product;

#[derive(Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Serialize)]
pub struct ProductResponse {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
}

impl From<Product> for ProductResponse {
    fn from(product: Product) -> Self {
        Self {
            id: product.id(),
            name: product.name().to_string(),
            description: product.description().to_string(),
            icon: product.icon().map(str::to_string),
        }
    }
}

impl From<&Product> for ProductResponse {
    fn from(product: &Product) -> Self {
        Self {
            id: product.id(),
            name: product.name().to_string(),
            description: product.description().to_string(),
            icon: product.icon().map(str::to_string),
        }
    }
}
