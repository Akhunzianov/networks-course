use std::sync::{
    Mutex, MutexGuard,
    atomic::{AtomicU64, Ordering},
};
use tracing::{error, info, warn};

use crate::{
    domain::product::Product,
    domain::repositories::ProductRepository,
    error::{AppError, AppResult},
};

pub struct InMemoryProductRepository {
    products: Mutex<Vec<Product>>,
    next_id: AtomicU64,
}

impl InMemoryProductRepository {
    pub fn new() -> Self {
        Self {
            products: Mutex::new(Vec::new()),
            next_id: AtomicU64::new(0),
        }
    }

    fn lock_products(&self) -> AppResult<MutexGuard<'_, Vec<Product>>> {
        self.products.lock().map_err(|_| {
            error!("product repo mutex poisoned");
            AppError::Internal
        })
    }
}

impl ProductRepository for InMemoryProductRepository {
    fn create(&self, name: String, description: String) -> AppResult<Product> {
        let mut products = self.lock_products()?;
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        info!(product_id = id, name = %name, "repo create");
        let product = Product::new(id, name, description)?;
        products.push(product.clone());
        Ok(product)
    }

    fn get(&self, id: u64) -> AppResult<Product> {
        let products = self.lock_products()?;
        match products.iter().find(|p| p.id() == id).cloned() {
            Some(product) => {
                info!(product_id = id, "repo get");
                Ok(product)
            }
            None => {
                warn!(product_id = id, "repo get not found");
                Err(AppError::NotFound)
            }
        }
    }

    fn update(&self, product: Product) -> AppResult<Product> {
        let mut products = self.lock_products()?;
        let res = match products.iter_mut().find(|p| p.id() == product.id()) {
            Some(res) => res,
            None => {
                warn!(product_id = product.id(), "repo update not found");
                return Err(AppError::NotFound);
            }
        };
        info!(product_id = product.id(), "repo update");
        *res = product.clone();
        Ok(product)
    }

    fn delete(&self, id: u64) -> AppResult<Product> {
        let mut products = self.lock_products()?;
        let pos = match products.iter().position(|p| p.id() == id) {
            Some(pos) => pos,
            None => {
                warn!(product_id = id, "repo delete not found");
                return Err(AppError::NotFound);
            }
        };
        info!(product_id = id, "repo delete");
        Ok(products.remove(pos))
    }

    fn list(&self) -> AppResult<Vec<Product>> {
        let products = self.lock_products()?;
        info!(count = products.len(), "repo list");
        Ok(products.clone())
    }
}
