use std::path::PathBuf;

use crate::error::{AppError, AppResult};

#[derive(Clone, Debug)]
pub struct Product {
    id: u64,
    name: String,
    description: String,
    icon: Option<String>,
}

impl Product {
    pub fn new(id: u64, name: String, description: String) -> AppResult<Self> {
        if name.trim().is_empty() {
            return Err(AppError::BadRequest("Product name cannot be empty".into()));
        }

        Ok(Self {
            id,
            name,
            description,
            icon: None,
        })
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    pub fn update(
        &mut self,
        name: Option<String>,
        description: Option<String>,
        icon: Option<PathBuf>,
    ) -> AppResult<()> {
        if let Some(name) = name {
            if name.trim().is_empty() {
                return Err(AppError::BadRequest("Product name cannot be empty".into()));
            }
            self.name = name;
        }

        if let Some(description) = description {
            self.description = description;
        }

        if let Some(icon) = icon {
            if icon.as_os_str().is_empty() {
                return Err(AppError::BadRequest("Icon name cannot be empty".into()));
            }
            self.icon = Some(
                icon.into_os_string()
                    .into_string()
                    .map_err(|_| AppError::Internal)?,
            );
        }

        Ok(())
    }
}
