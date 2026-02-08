pub struct Config {
    pub user_agents: Vec<String>,
    pub timeout_seconds: u64,
    pub min_delay_ms: u64,
    pub max_delay_ms: u64,
    pub max_retries: u32,
    pub base_url: String,
    pub concurrent_requests: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            user_agents: vec![
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15".to_string(),
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:122.0) Gecko/20100101 Firefox/122.0".to_string(),
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            ],
            timeout_seconds: 30,
            min_delay_ms: 2000, // Minimum 2 seconds
            max_delay_ms: 5000, // Maximum 5 seconds
            max_retries: 3,
            base_url: "https://www.gsmarena.com".to_string(),
            concurrent_requests: 2, // Safe default to avoid blocks
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrent_requests = concurrency;
        self
    }
}