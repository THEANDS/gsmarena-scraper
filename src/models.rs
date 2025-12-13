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
    
    // Display information
    pub display_ratio: Option<String>,
    pub display_area_cm2: Option<String>,
    pub resolution: Option<String>,
    pub screen_size: Option<String>,
    pub ppi: Option<String>,
    
    // Other specs (para futuras expansões)
    pub os: Option<String>,
    pub chipset: Option<String>,
    pub ram: Option<String>,
    pub storage: Option<String>,
    pub battery: Option<String>,
    
    pub status_code: u16,
    pub error_message: Option<String>,
}

impl PhoneDetails {
    pub fn new(phone: &Phone) -> Self {
        Self {
            phone_id: phone.id,
            model: phone.model.clone(),
            url: phone.url.clone(),
            display_ratio: None,
            display_area_cm2: None,
            resolution: None,
            screen_size: None,
            ppi: None,
            os: None,
            chipset: None,
            ram: None,
            storage: None,
            battery: None,
            status_code: 0,
            error_message: None,
        }
    }
    
    pub fn has_display_info(&self) -> bool {
        self.display_ratio.is_some() && self.display_area_cm2.is_some()
    }
}