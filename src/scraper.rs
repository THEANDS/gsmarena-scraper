use chrono::Local;
use reqwest::blocking::Client;
use select::document::Document;
use select::predicate::{Name, Class, Predicate};
use std::collections::HashSet;
use std::thread;
use std::time::{Duration, Instant};
use regex::Regex;
use std::fs::{File, OpenOptions};
use std::io::Write;

use crate::models::{Phone, PhoneDetails};
use crate::config::Config;
pub struct PhoneScraper {
    client: Client,
    config: Config,
}

impl PhoneScraper {
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::builder()
            .user_agent(&config.user_agent)
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;
        
        Ok(Self { client, config })
    }
    
    // Função 1: Coletar URLs de smartphones
   pub fn scrape_phone_urls(&self, max_pages: Option<usize>) -> Result<Vec<Phone>, Box<dyn std::error::Error>> {
    println!("🚀 Coletando URLs de smartphones Samsung...");
    
    let mut all_phones = Vec::new();
    let mut visited_urls = HashSet::new();
    let mut current_page_num = 1;
    let max_pages = max_pages.unwrap_or(3); // Padrão: 9 páginas
    let mut has_more_pages = true;
    
    println!("📖 Limite: {} páginas (celulares mais recentes)", max_pages);
    
    while has_more_pages && current_page_num <= max_pages {
        let page_url = if current_page_num == 1 {
            "https://www.gsmarena.com/apple-phones-48.php".to_string()
        } else {
            format!("https://www.gsmarena.com/apple-phones-f-48-0-p{}.php", current_page_num)
        };
        
        if visited_urls.contains(&page_url) {
            println!("⏭️  Página {} já visitada", current_page_num);
            break;
        }
        
        println!("📄 Página {}/{}: {}", current_page_num, max_pages, page_url);
        visited_urls.insert(page_url.clone());
        
        match self.client.get(&page_url).send() {
            Ok(response) => {
                if response.status().is_success() {
                    let body = response.text()?;
                    let document = Document::from(body.as_str());
                    
                    let phones_from_page = self.extract_phones_from_page(&document);
                    println!("   ✅ {} smartphones encontrados", phones_from_page.len());
                    
                    // Adicionar IDs
                    let start_id = all_phones.len() + 1;
                    let phones_with_ids: Vec<Phone> = phones_from_page
                        .into_iter()
                        .enumerate()
                        .map(|(i, (model, url))| Phone {
                            id: start_id + i,
                            model,
                            url,
                            status: "pending".to_string(),
                        })
                        .collect();
                    
                    all_phones.extend(phones_with_ids);
                    
                    // Verificar próxima página, mas respeitar limite
                    has_more_pages = self.has_next_page(&document, current_page_num);
                    
                    if has_more_pages && current_page_num < max_pages {
                        current_page_num += 1;
                        self.random_delay();
                    } else {
                        println!("⏹️  Limite de páginas alcançado: {}", current_page_num);
                        has_more_pages = false;
                    }
                } else {
                    println!("❌ Erro HTTP {}", response.status());
                    has_more_pages = false;
                }
            }
            Err(e) => {
                println!("❌ Erro: {}", e);
                has_more_pages = false;
            }
        }
    }
    
    // Remover duplicatas
    all_phones.sort_by(|a, b| a.url.cmp(&b.url));
    all_phones.dedup_by(|a, b| a.url == b.url);
    
    // Reatribuir IDs após dedup
    for (i, phone) in all_phones.iter_mut().enumerate() {
        phone.id = i + 1;
    }
    
    println!("✅ Total de smartphones (páginas 1-{}): {}", max_pages, all_phones.len());
    Ok(all_phones)
}
    
    // Função 2: Extrair detalhes de cada smartphone
    pub fn scrape_phone_details(&self, phones: &[Phone]) -> Result<Vec<PhoneDetails>, Box<dyn std::error::Error>> {
        println!("\n📱 Iniciando extração de detalhes...");
        println!("📊 {} telefones para processar", phones.len());
        
        let mut details = Vec::new();
        let total = phones.len();
        
        for (index, phone) in phones.iter().enumerate() {
            let current = index + 1;
            println!("\n[{}/{}] Processando: {}", current, total, phone.model);
            
            let detail = self.scrape_single_phone_details(phone, current, total);
            details.push(detail);
            
            if current < total {
                self.random_delay();
            }
        }
        
        Ok(details)
    }
    
    // Métodos auxiliares privados
    
    
    fn extract_phones_from_page(&self, document: &Document) -> Vec<(String, String)> {
        let mut phones = Vec::new();
        
        for node in document.find(Class("makers").descendant(Name("a"))) {
            if let Some(href) = node.attr("href") {
                if href.ends_with(".php") && !href.contains("review") && !href.contains("#") {
                    let full_url = format!("{}/{}", self.config.base_url, href);
                    let phone_name = self.extract_phone_name_from_node(&node);
                    
                    if self.is_smartphone(&phone_name) {
                        phones.push((phone_name, full_url));
                    }
                }
            }
        }
        
        phones
    }
    
    fn extract_phone_name_from_node(&self, node: &select::node::Node) -> String {
        node.find(Name("strong"))
            .next()
            .or_else(|| node.find(Name("span")).next())
            .map(|n| n.text().trim().to_string())
            .unwrap_or_else(|| {
                node.attr("href")
                    .map(|href| self.extract_phone_name_from_url(href))
                    .unwrap_or_else(|| "Desconhecido".to_string())
            })
    }
    
    fn extract_phone_name_from_url(&self, href: &str) -> String {
        let filename = href.split('/').last().unwrap_or(href);
        let without_ext = filename.trim_end_matches(".php");
        let model_part = without_ext.split('-').next().unwrap_or("");
        
        model_part
            .split('_')
            .skip(1)
            .map(|word| {
                let mut chars: Vec<char> = word.chars().collect();
                if !chars.is_empty() {
                    chars[0] = chars[0].to_uppercase().next().unwrap();
                }
                chars.into_iter().collect()
            })
            .collect::<Vec<String>>()
            .join(" ")
    }
    
   fn is_smartphone(&self, name: &str) -> bool {
    let lower = name.to_lowercase();
    
    // Excluir definitivamente não-smartphones
    let excluded_keywords = [
        "tab", "tablet", "pad",           // Tablets
        "watch", "gear", "wear",          // Smartwatches
        "active", "xcover", "rugged",     // Rugged/enterprise
        "buds", "level", "icon",          // Acessórios
        "galaxy z fold",                  // Dobráveis (opcional)
        "galaxy z flip",                  // Flip (opcional)
        "galaxy note",                    // Notes (alguns são antigos)
    ];
    
    // Verificar se contém alguma palavra excluída
    for keyword in &excluded_keywords {
        if lower.contains(keyword) {
            return false;
        }
    }
    
    // Incluir apenas modelos modernos (opcional)
    let include_patterns = [
        "galaxy s", "galaxy a", "galaxy m", "galaxy f", "galaxy j"
    ];
    
    // Verificar se é um modelo de smartphone reconhecido
    let is_known_model = include_patterns.iter().any(|pattern| lower.contains(pattern));
    
    // Filtro por ano (aproximado) - excluir modelos muito antigos
    let exclude_old_models = [
        "galaxy s2", "galaxy s3", "galaxy s4", "galaxy s5",
        "galaxy note 2", "galaxy note 3", "galaxy note 4",
        "galaxy ace", "galaxy core", "galaxy young",
        "galaxy grand", "galaxy mega",
    ];
    
    let is_old_model = exclude_old_models.iter().any(|model| lower.contains(model));
    
    // Critérios finais
    name.trim().len() > 0 &&
    name != "Desconhecido" &&
    !is_old_model &&
    (is_known_model || !lower.contains("galaxy")) // Se não for Galaxy, aceitar se passar outros filtros
}
    
    fn has_next_page(&self, document: &Document, current_page: usize) -> bool {
        // Verificar por link da próxima página
        for node in document.find(Name("a")) {
            if let Some(href) = node.attr("href") {
                if href.contains(&format!("-p{}.php", current_page + 1)) {
                    return true;
                }
                
                let text = node.text().to_lowercase();
                if text.contains("next") || text.contains(">") {
                    return true;
                }
            }
        }
        false
    }
    
    fn scrape_single_phone_details(&self, phone: &Phone, current: usize, total: usize) -> PhoneDetails {
        let mut details = PhoneDetails::new(phone);
        
        println!("   📍 URL: {}", phone.url);
        
        match self.client.get(&phone.url).send() {
            Ok(response) => {
                details.status_code = response.status().as_u16();
                
                if response.status().is_success() {
                    match response.text() {
                        Ok(html) => {
                            self.extract_display_info(&html, &mut details);
                            println!("   ✅ Dados extraídos");
                        }
                        Err(e) => {
                            details.error_message = Some(format!("Erro ao ler resposta: {}", e));
                            println!("   ❌ Erro: {}", e);
                        }
                    }
                } else {
                    details.error_message = Some(format!("HTTP {}", response.status()));
                    println!("   ❌ HTTP Error: {}", response.status());
                }
            }
            Err(e) => {
                details.error_message = Some(format!("Request error: {}", e));
                println!("   ❌ Erro: {}", e);
            }
        }
        
        details
    }
    
    fn extract_display_info(&self, html: &str, details: &mut PhoneDetails) {
        let document = Document::from(html);
        
        // Procurar na tabela de especificações
        for node in document.find(Name("tr")) {
            let cells: Vec<String> = node.find(Name("td"))
                .map(|td| td.text().trim().to_string())
                .collect();
            
            if cells.len() >= 2 {
                let label = cells[0].to_lowercase();
                let value = cells[1].clone();
                
                match label.as_str() {
                    l if l.contains("size") => {
                        details.screen_size = Some(self.extract_screen_size(&value));
                        
                        // Calcular área se tivermos ratio
                        if let (Some(size), Some(ratio)) = (&details.screen_size, &details.display_ratio) {
                            if let Some(area) = self.calculate_display_area(size, ratio) {
                                details.display_area_cm2 = Some(format!("{:.2}", area));
                            }
                        }
                    }
                    l if l.contains("resolution") => {
                        details.resolution = Some(self.extract_resolution(&value));
                    }
                    l if l.contains("ratio") => {
                        details.display_ratio = Some(self.extract_ratio(&value));
                        
                        // Calcular área se tivermos tamanho
                        if let (Some(size), Some(ratio)) = (&details.screen_size, &details.display_ratio) {
                            if let Some(area) = self.calculate_display_area(size, ratio) {
                                details.display_area_cm2 = Some(format!("{:.2}", area));
                            }
                        }
                    }
                    l if l.contains("ppi") || l.contains("pixel density") => {
                        details.ppi = Some(self.extract_ppi(&value));
                    }
                    _ => {}
                }
            }
        }
        
        // Tentar extrair por regex se não encontrou na tabela
        if details.screen_size.is_none() || details.display_ratio.is_none() {
            self.extract_with_regex(html, details);
        }
    }
    
    fn extract_with_regex(&self, html: &str, details: &mut PhoneDetails) {
        let html_lower = html.to_lowercase();
        
        if details.screen_size.is_none() {
            if let Some(size) = self.find_pattern(&html_lower, r#"(\d+\.?\d*)\s*(?:inches|"|inch)"#) {
                details.screen_size = Some(format!("{}\"", size));
            }
        }
        
        if details.display_ratio.is_none() {
            if let Some(ratio) = self.find_pattern(&html_lower, r"(\d+\.?\d*\s*[:]\s*\d+\.?\d*)") {
                details.display_ratio = Some(ratio.replace(" ", ""));
            }
        }
        
        if details.resolution.is_none() {
            if let Some(res) = self.find_pattern(&html_lower, r"(\d+\s*x\s*\d+)") {
                details.resolution = Some(res.replace(" ", ""));
            }
        }
        
        // Calcular área se agora temos ambos
        if let (Some(size), Some(ratio)) = (&details.screen_size, &details.display_ratio) {
            if let Some(area) = self.calculate_display_area(size, ratio) {
                details.display_area_cm2 = Some(format!("{:.2}", area));
            }
        }
    }
    
    fn extract_screen_size(&self, text: &str) -> String {
        Regex::new(r#"(\d+\.?\d*)\s*(?:inches|"|inch)"#)
            .ok()
            .and_then(|re| re.captures(text))
            .map(|cap| format!("{}\"", &cap[1]))
            .unwrap_or_else(|| "N/A".to_string())
    }
    
    fn extract_ratio(&self, text: &str) -> String {
        Regex::new(r"(\d+\.?\d*\s*[:]\s*\d+\.?\d*)")
            .ok()
            .and_then(|re| re.captures(text))
            .map(|cap| cap[1].replace(" ", ""))
            .unwrap_or_else(|| "N/A".to_string())
    }
    
    fn extract_resolution(&self, text: &str) -> String {
        Regex::new(r"(\d+\s*x\s*\d+)")
            .ok()
            .and_then(|re| re.captures(text))
            .map(|cap| cap[1].replace(" ", ""))
            .unwrap_or_else(|| "N/A".to_string())
    }
    
fn extract_ppi(&self, text: &str) -> String {
    let lower = text.to_lowercase();

    Regex::new(r"(\d+\.?\d*)\s*ppi")
        .ok()
        .and_then(|re| re.captures(&lower))
        .map(|cap| format!("{} ppi", &cap[1]))
        .unwrap_or_else(|| "N/A".to_string())
}
    fn find_pattern(&self, text: &str, pattern: &str) -> Option<String> {
        Regex::new(pattern)
            .ok()?
            .captures(text)
            .map(|cap| cap[1].to_string())
    }
    
    fn calculate_display_area(&self, size_text: &str, ratio_text: &str) -> Option<f64> {
        let size_re = Regex::new(r"(\d+\.?\d*)").ok()?;
        let size_cap = size_re.captures(size_text)?;
        let diagonal_inches: f64 = size_cap[1].parse().ok()?;
        
        let ratio_re = Regex::new(r"(\d+\.?\d*)\s*[:]\s*(\d+\.?\d*)").ok()?;
        let ratio_cap = ratio_re.captures(ratio_text)?;
        
        let a: f64 = ratio_cap[1].parse().ok()?;
        let b: f64 = ratio_cap[2].parse().ok()?;
        
        let ratio = b / a;
        let width_inches = diagonal_inches / (1.0 + ratio * ratio).sqrt();
        let height_inches = width_inches * ratio;
        
        let width_cm = width_inches * 2.54;
        let height_cm = height_inches * 2.54;
        
        Some(width_cm * height_cm)
    }
    
  
        pub fn scrape_phone_details_with_batching(&self, phones: &[Phone]) -> Result<Vec<PhoneDetails>, Box<dyn std::error::Error>> {
        println!("📱 Iniciando extração de detalhes com sistema de batching...");
        println!("📊 Total de telefones: {}", phones.len());
        println!("⚙️  Configuração: {} por lote, pausa de {}s após {} lotes",
            self.config.batch_size,
            self.config.pause_duration_seconds,
            self.config.pause_after_batches
        );
        
        let mut all_details = Vec::new();
        let total_batches = (phones.len() + self.config.batch_size - 1) / self.config.batch_size;
        
        // Verificar se já existe progresso salvo
        let (start_batch, processed_count) = self.load_progress()?;
        
        if processed_count > 0 {
            println!("🔄 Retomando do batch {} ({} já processados)", start_batch, processed_count);
        }
        
        for batch_num in start_batch..total_batches {
            let start_idx = batch_num * self.config.batch_size;
            let end_idx = std::cmp::min(start_idx + self.config.batch_size, phones.len());
            let batch = &phones[start_idx..end_idx];
            
            println!("\n═══════════════════════════════════════════════════");
            println!("📦 BATCH {}/{} (Telefones {}-{})", 
                batch_num + 1, total_batches, start_idx + 1, end_idx);
            println!("═══════════════════════════════════════════════════");
            
            let batch_details = self.process_batch(batch, batch_num + 1)?;
            all_details.extend(batch_details);
            
            // Salvar progresso após cada batch
            self.save_progress(batch_num + 1, start_idx + batch.len())?;
            
            // Verificar se precisa pausar
            if (batch_num + 1) % self.config.pause_after_batches == 0 {
                println!("\n⏸️  PAUSA: {} requisições completadas", 
                    (batch_num + 1) * self.config.batch_size);
                println!("   Aguardando {} segundos para evitar limitação de taxa...", 
                    self.config.pause_duration_seconds);
                
                self.countdown_pause(self.config.pause_duration_seconds);
                
                println!("▶️  Retomando processamento...");
            }
            
            // Pequena pausa entre batches
            if batch_num < total_batches - 1 {
                let batch_pause = 5 + rand::random::<u64>() % 10;
                println!("   ⏳ Pausa entre batches: {} segundos", batch_pause);
                thread::sleep(Duration::from_secs(batch_pause));
            }
        }
        
      
        
        println!("\n✅ Processamento concluído! {} detalhes extraídos.", all_details.len());
        
        Ok(all_details)
    }
    
    fn process_batch(&self, batch: &[Phone], batch_num: usize) -> Result<Vec<PhoneDetails>, Box<dyn std::error::Error>> {
        let mut batch_details = Vec::new();
        let batch_start_time = Instant::now();
        
        for (i, phone) in batch.iter().enumerate() {
            let global_index = (batch_num - 1) * self.config.batch_size + i + 1;
            
            println!("\n[{}/?] Batch {} - Item {}: {}", 
                global_index, batch_num, i + 1, phone.model);
            println!("   📍 URL: {}", phone.url);
            
            let detail = self.scrape_single_phone_with_retry(phone)?;
            batch_details.push(detail);
            
            // Delay entre requisições no mesmo batch
            if i < batch.len() - 1 {
                self.random_delay();
            }
        }
        
        let batch_duration = batch_start_time.elapsed();
        println!("\n   ✅ Batch {} concluído em {:.2?}", batch_num, batch_duration);
        
        Ok(batch_details)
    }
    
    fn scrape_single_phone_with_retry(&self, phone: &Phone) -> Result<PhoneDetails, Box<dyn std::error::Error>> {
        let mut detail = PhoneDetails::new(phone);
        
        for attempt in 1..=self.config.max_retries {
            match self.client.get(&phone.url).send() {
                Ok(response) => {
                    detail.status_code = response.status().as_u16();
                    
                    if response.status().is_success() {
                        match response.text() {
                            Ok(html) => {
                                self.extract_display_info(&html, &mut detail);
                                if detail.has_display_info() {
                                    println!("   ✅ Dados extraídos (tentativa {})", attempt);
                                } else {
                                    println!("   ⚠️  Informações parciais");
                                }
                                return Ok(detail);
                            }
                            Err(e) => {
                                if attempt == self.config.max_retries {
                                    detail.error_message = Some(format!("Erro ao ler resposta: {}", e));
                                    println!("   ❌ Falha final: {}", e);
                                } else {
                                    println!("   🔄 Tentativa {}/{} falhou, tentando novamente...", 
                                        attempt, self.config.max_retries);
                                    thread::sleep(Duration::from_secs(2 * attempt as u64));
                                }
                            }
                        }
                    } else if response.status() == 429 {
                        // Too Many Requests - pausa mais longa
                        println!("   ⚠️  Erro 429 (Too Many Requests)");
                        println!("   🕐 Aguardando 30 segundos antes de retry...");
                        thread::sleep(Duration::from_secs(30));
                        continue;
                    } else {
                        detail.error_message = Some(format!("HTTP {}", response.status()));
                        println!("   ❌ HTTP Error: {}", response.status());
                        return Ok(detail);
                    }
                }
                Err(e) => {
                    if attempt == self.config.max_retries {
                        detail.error_message = Some(format!("Request error: {}", e));
                        println!("   ❌ Falha final: {}", e);
                        return Ok(detail);
                    } else {
                        println!("   🔄 Tentativa {}/{} falhou: {}, tentando novamente...", 
                            attempt, self.config.max_retries, e);
                        thread::sleep(Duration::from_secs(3 * attempt as u64));
                    }
                }
            }
        }
        
        Ok(detail)
    }
    
    fn load_progress(&self) -> Result<(usize, usize), Box<dyn std::error::Error>> {
        let progress_file = "scraper_progress.txt";
        
        if !std::path::Path::new(progress_file).exists() {
            return Ok((0, 0));
        }
        
        let content = std::fs::read_to_string(progress_file)?;
        let lines: Vec<&str> = content.trim().split('\n').collect();
        
        if lines.len() >= 2 {
            let batch_num = lines[0].parse().unwrap_or(0);
            let processed = lines[1].parse().unwrap_or(0);
            Ok((batch_num, processed))
        } else {
            Ok((0, 0))
        }
    }
    
    fn save_progress(&self, batch_num: usize, processed_count: usize) -> Result<(), Box<dyn std::error::Error>> {
        let progress_file = "scraper_progress.txt";
        let mut file = File::create(progress_file)?;
        
        writeln!(file, "{}", batch_num)?;
        writeln!(file, "{}", processed_count)?;
        writeln!(file, "{}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"))?;
        
        Ok(())
    }
  
    fn countdown_pause(&self, seconds: u64) {
        for i in (1..=seconds).rev() {
            print!("\r   ⏱️  Retomando em {} segundos...", i);
            std::io::stdout().flush().unwrap();
            thread::sleep(Duration::from_secs(1));
        }
        println!("\r   ✅ Pausa concluída!                    ");
    }
    
    fn random_delay(&self) {
        let base_delay = self.config.delay_between_requests_ms;
        let jitter = rand::random::<u64>() % 500;
        let total_delay = base_delay + jitter;
        
        println!("   ⏳ Delay: {}ms", total_delay);
        thread::sleep(Duration::from_millis(total_delay));
    }

      
    pub fn scrape_one_per_minute(&self, phones: &[Phone]) -> Result<Vec<PhoneDetails>, Box<dyn std::error::Error>> {
        println!("📱 Iniciando extração (1 por minuto)...");
        println!("📊 Total de telefones: {}", phones.len());
        println!("⏰ Configuração: 1 requisição por minuto");
        
        let mut all_details = Vec::new();
        let total = phones.len();
        let current_time = Local::now();
        
        // Carregar progresso se existir
        let progress_file = "one_per_minute_progress_iphone.json";
        let results_file = "one_per_minute_results_iphone.csv";
        let mut start_index = 0;
        
        // Criar ou abrir arquivo de resultados
        let mut results_writer = if std::path::Path::new(results_file).exists() {
            OpenOptions::new()
                .append(true)
                .open(results_file)?
        } else {
            let mut file = File::create(results_file)?;
            // Escrever cabeçalho
            writeln!(file, "ID,Modelo,URL,Ratio,Area_cm2,Resolução,Tamanho,PPI,Status,Extraído_em")?;
            file
        };
        
        // Criar arquivo de log
        let mut log_file = File::create("one_per_minute_progress_iphone_log.txt")?;
        writeln!(log_file, "🚀 INÍCIO DO PROCESSAMENTO (1 POR MINUTO)")?;
        writeln!(log_file, "Data: {}", Local::now().format("%Y-%m-%d %H:%M:%S"))?;
        writeln!(log_file, "Total de telefones: {}", total)?;
        writeln!(log_file, "{}", "=".repeat(60))?;
        
        if std::path::Path::new(progress_file).exists() {
            if let Ok(progress) = std::fs::read_to_string(progress_file) {
                if let Ok(last_index) = progress.trim().parse::<usize>() {
                    start_index = last_index;
                    println!("🔄 Continuando do telefone {} de {}", start_index + 1, total);
                    writeln!(log_file, "🔄 Retomando do telefone {} de {}", start_index + 1, total)?;
                }
            }
        }
        
        let mut successful_count = 0;
        let mut failed_count = 0;
        
        for (index, phone) in phones.iter().enumerate().skip(start_index) {
            let current = index + 1;

            
            println!("\n═══════════════════════════════════════════════════");
            println!("⏰ [{}/{}] HORA: {}", current, total, current_time.format("%H:%M:%S"));
            println!("📱 PROCESSANDO: {}", phone.model);
            println!("📍 URL: {}", phone.url);
            
            // Registrar no log
            writeln!(log_file, "\n[{}/{}] {} - {}", 
                current, total, current_time.format("%H:%M:%S"), phone.model)?;
            
            let detail = self.scrape_single_phone_with_retry(phone)?;
            all_details.push(detail.clone());
            
            // SALVAR CADA RESULTADO IMEDIATAMENTE
            self.save_single_result(&detail, &mut results_writer)?;
            
            // Atualizar contadores
            if detail.has_display_info() {
                successful_count += 1;
                writeln!(log_file, "   ✅ Sucesso - Ratio: {}, Área: {} cm²", 
                    detail.display_ratio.as_deref().unwrap_or("N/A"),
                    detail.display_area_cm2.as_deref().unwrap_or("N/A"))?;
            } else {
                failed_count += 1;
                if let Some(error) = &detail.error_message {
                    writeln!(log_file, "   ❌ Falha: {}", error)?;
                } else {
                    writeln!(log_file, "   ❌ Falha: Informações não encontradas")?;
                }
            }
            
            // Salvar progresso
            if let Ok(mut file) = File::create(progress_file) {
                writeln!(file, "{}", index)?;
            }
            
            // Mostrar estatísticas
            println!("📊 PROGRESSO: {}/{} ({}✅ {}❌)", current, total, successful_count, failed_count);
            
            // Calcular tempo restante
            let remaining = total - current;
            if remaining > 0 {
                let remaining_minutes = remaining;
                let remaining_hours = remaining_minutes as f32 / 60.0;
                
                println!("⏳ Tempo restante: {} minutos ({:.1} horas)", remaining_minutes, remaining_hours);
                
                // DELAY DE 1 MINUTO
                println!("\n⏸️  AGUARDANDO 1 MINUTO PARA PRÓXIMO...");
                
                // Contagem regressiva
                self.countdown_one_minute();
            }
        }
        
        // Finalizar e salvar resumo
        writeln!(log_file, "\n{}", "=".repeat(60))?;
        writeln!(log_file, "Sucessos: {}, Falhas: {}", successful_count, failed_count)?;
        writeln!(log_file, "Taxa de sucesso: {:.1}%", 
                (successful_count as f32 / total as f32) * 100.0)?;
        
        // Limpar arquivo de progresso
        let _ = std::fs::remove_file(progress_file);
        
        println!("\n✅ Processamento concluído!");
        println!("📁 Resultados salvos em: {}", results_file);
        println!("📝 Log salvo em: oone_per_minute_progress_iphone_log.txt");
        println!("📊 RESUMO: {}✅ {}❌ ({:.1}% sucesso)", 
                successful_count, failed_count,
                (successful_count as f32 / total as f32) * 100.0);
        
        Ok(all_details)
    }
    
    // Nova função para salvar cada resultado individualmente
    fn save_single_result(&self, detail: &PhoneDetails, writer: &mut File) -> Result<(), Box<dyn std::error::Error>> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        
        writeln!(writer, "{},\"{}\",{},{},{},{},{},{},{},{}",
            detail.phone_id,
            detail.model.replace("\"", "\"\""),
            detail.url,
            detail.display_ratio.as_deref().unwrap_or("N/A"),
            detail.display_area_cm2.as_deref().unwrap_or("N/A"),
            detail.resolution.as_deref().unwrap_or("N/A"),
            detail.screen_size.as_deref().unwrap_or("N/A"),
            detail.ppi.as_deref().unwrap_or("N/A"),
            if detail.has_display_info() { "SUCCESS" } else { "FAILED" },
            timestamp
        )?;
        
        writer.flush()?; // Forçar escrita imediata
        Ok(())
    }
    
    // Contagem regressiva de 1 minuto
    fn countdown_one_minute(&self) {
        for second in (1..=60).rev() {
            if second % 10 == 0 || second <= 5 {
                print!("\r   ⏱️  Próximo em {:02} segundos", second);
                std::io::stdout().flush().unwrap();
            }
            thread::sleep(Duration::from_secs(1));
        }
        println!("\r   ✅ Próximo processamento!                    ");
    }
   
}