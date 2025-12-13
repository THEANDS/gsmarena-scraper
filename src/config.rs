pub struct Config {
    pub user_agent: String,
    pub timeout_seconds: u64,
    pub delay_between_requests_ms: u64,
    pub max_retries: u32,
    pub base_url: String,
    pub batch_size: usize,          // Novo: processar em lotes
    pub pause_after_batches: usize, // Novo: pausar após X lotes
    pub pause_duration_seconds: u64, // Novo: duração da pausa
}

impl Default for Config {
    fn default() -> Self {
        Self {
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
            timeout_seconds: 30,
            delay_between_requests_ms: 1500,  // 1.5s entre requisições
            max_retries: 3,
            base_url: "https://www.gsmarena.com".to_string(),
            batch_size: 20,          // Processar 20 em 20
            pause_after_batches: 5,  // Pausar após 5 batches (100 requests)
            pause_duration_seconds: 120, // 2 minutos de pausa
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_between_requests_ms = delay_ms;
        self
    }
    
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }
    
    pub fn with_pause_settings(mut self, pause_after_batches: usize, pause_seconds: u64) -> Self {
        self.pause_after_batches = pause_after_batches;
        self.pause_duration_seconds = pause_seconds;
        self
    }
}