use crate::config::Config;
use crate::services::product::ProductService;

#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)]
    pub config: Config,

    pub product_service: ProductService,
}

impl AppState {
    pub fn new(config: Config, product_service: ProductService) -> Self {
        Self {
            config,
            product_service,
        }
    }
}
