use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phone {
    pub id: usize,
    pub model: String,
    pub url: String,
    pub status: String, // "pending", "processed", "error"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneDetails {
    pub phone_id: usize,
    pub model: String,
    pub url: String,
    
    // Body - Critical for cases/protectors
    pub body_dimensions: Option<String>,
    pub body_weight: Option<String>,
    pub body_build: Option<String>,
    pub sim: Option<String>,
    
    // Display - Critical for screen protectors
    pub display_type: Option<String>,
    pub display_size: Option<String>,
    pub resolution: Option<String>,
    pub protection: Option<String>, // Gorilla Glass, etc.
    
    pub status_code: u16,
    pub error_message: Option<String>,
}

impl PhoneDetails {
    pub fn new(phone: &Phone) -> Self {
        Self {
            phone_id: phone.id,
            model: phone.model.clone(),
            url: phone.url.clone(),
            body_dimensions: None,
            body_weight: None,
            body_build: None,
            sim: None,
            display_type: None,
            display_size: None,
            resolution: None,
            protection: None,
            status_code: 0,
            error_message: None,
        }
    }
    
    pub fn has_specs(&self) -> bool {
        self.body_dimensions.is_some() || self.display_size.is_some()
    }
}