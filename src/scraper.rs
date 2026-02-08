use std::collections::HashSet;
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::sleep;
use reqwest::Client;
use select::document::Document;
use select::predicate::{Name, Class, Predicate, Attr};
use rand::Rng;

use crate::models::{Phone, PhoneDetails};
use crate::config::Config;

pub struct PhoneScraper {
    client: Client,
    config: Config,
    semaphore: Arc<Semaphore>,
}

impl PhoneScraper {
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;
            
        let semaphore = Arc::new(Semaphore::new(config.concurrent_requests));
        
        Ok(Self { 
            client, 
            config,
            semaphore 
        })
    }
    
    /// Get a random User-Agent from the config
    fn get_random_user_agent(&self) -> &str {
        let mut rng = rand::rng();
        let idx = rng.random_range(0..self.config.user_agents.len());
        &self.config.user_agents[idx]
    }
    
    /// Async delay with jitter
    async fn random_delay(&self) {
        let delay = {
            let mut rng = rand::rng();
            rng.random_range(self.config.min_delay_ms..=self.config.max_delay_ms)
        };
        sleep(Duration::from_millis(delay)).await;
    }

    /// Crawl for phone URLs
    pub async fn scrape_phone_urls(&self, brand_url: &str, max_pages: usize) -> Result<Vec<Phone>, Box<dyn std::error::Error>> {
        println!("🚀 Iniciando coleta de URLs de: {}", brand_url);
        
        let mut all_phones = Vec::new();
        let mut visited_urls = HashSet::new();
        let mut current_page_num = 1;
        
        // Base URL parsing for pagination
        // Assuming format like https://www.gsmarena.com/realme-phones-118.php
        let base_parts: Vec<&str> = brand_url.split(".php").collect();
        let url_prefix = base_parts[0]; // e.g., .../realme-phones-118
        
        while current_page_num <= max_pages {
            let page_url = if current_page_num == 1 {
                brand_url.to_string()
            } else {
                // Restaurar lógica específica para Realme se detectado, para garantir compatibilidade
                if brand_url.contains("realme-phones-118") {
                    format!("https://www.gsmarena.com/realme-phones-f-118-0-p{}.php", current_page_num)
                } else if url_prefix.contains("-phones-") {
                    // Lógica genérica para outras marcas
                    let parts: Vec<&str> = url_prefix.split("-phones-").collect();
                    if parts.len() == 2 {
                        format!("{}-phones-f-{}-0-p{}.php", parts[0], parts[1], current_page_num)
                    } else {
                         // Fallback simples
                        println!("⚠️  Não foi possível gerar URL de paginação para {}. Tentando padrão.", brand_url);
                        break;
                    }
                } else {
                    break;
                }
            };
            
            if visited_urls.contains(&page_url) {
                break;
            }
            
            println!("📄 Processando página {}: {}", current_page_num, page_url);
            visited_urls.insert(page_url.clone());
            
            let response = self.client.get(&page_url)
                .header("User-Agent", self.get_random_user_agent())
                .send()
                .await?;
                
            if !response.status().is_success() {
                println!("❌ Erro na página {}: {}", current_page_num, response.status());
                break;
            }
            
            let html = response.text().await?;
            let document = Document::from(html.as_str());
            
            let phones_found = self.extract_phones_from_page(&document);
            
            if phones_found.is_empty() {
                println!("⚠️  Nenhum telefone encontrado na página {}. Parando.", current_page_num);
                break;
            }
            
            println!("   ✅ Encontrados {} modelos", phones_found.len());
            all_phones.extend(phones_found);
            
            // Check for "Next" page just in case
            // If no next button, break
            // let has_next = document.find(Class("pages-next")).next().is_some();
            // if !has_next && current_page_num < max_pages {
            //      println!("⏹️  Fim da paginação detectado (pages-next não encontrado).");
            //      // break; // REMOVIDO: Forçar a continuação se o usuário pediu X páginas
            // }
            
            // Debug para entender o que está acontecendo
            // println!("   DEBUG: Próxima página? {}", has_next);
            
            current_page_num += 1;
            self.random_delay().await;
        }
        
        // Add IDs
        let final_phones: Vec<Phone> = all_phones.into_iter().enumerate().map(|(i, (model, url))| {
            Phone {
                id: i + 1,
                model,
                url,
                status: "pending".to_string(),
            }
        }).collect();
        
        Ok(final_phones)
    }
    
    fn extract_phones_from_page(&self, document: &Document) -> Vec<(String, String)> {
        let mut phones = Vec::new();
        
        for node in document.find(Class("makers").descendant(Name("a"))) {
            if let Some(href) = node.attr("href") {
                let full_url = format!("{}/{}", self.config.base_url, href);
                // Extract clean name
                let name = node.find(Name("span")).next().map(|n| n.text()).unwrap_or_else(|| "Unknown".to_string());
                
                // Clean text
                let name = name.replace("<br>", " ").trim().to_string();
                
                if self.is_smartphone(&name) {
                    phones.push((name, full_url));
                }
            }
        }
        
        phones
    }
    
    fn is_smartphone(&self, name: &str) -> bool {
        let lower = name.to_lowercase();
        // Palavras para excluir baseadas no pedido do usuário (relogio, tablet, smartwatch)
        // Site em inglês, então usamos termos em inglês
        let excluded_keywords = [
            "watch", "gear", "fit", "band", // Relógios/Pulseiras
            "tab", "pad", "tablet",         // Tablets
            "buds", "airpod", "freebuds"    // Fones (caso apareçam)
        ];
        
        for keyword in excluded_keywords {
            if lower.contains(keyword) {
                return false;
            }
        }
        
        true
    }

    /// Main async function to scrape all details with concurrency control
    pub async fn scrape_phone_details(&self, phones: &[Phone]) -> Result<Vec<PhoneDetails>, Box<dyn std::error::Error>> {
        println!("🚀 Iniciando extração de detalhes para {} telefones...", phones.len());
        println!("⚙️  Concorrência: {} workers", self.config.concurrent_requests);
        
        let mut tasks = Vec::new();
        // Removed unused total
        
        let mut details = Vec::new();
        
        for phone in phones {
            let permit = self.semaphore.clone().acquire_owned().await?;
            let client = self.client.clone();
            let url = phone.url.clone();
            let model = phone.model.clone();
            let phone_id = phone.id;
            let ua = self.get_random_user_agent().to_string();
            let delay_min = self.config.min_delay_ms;
            let delay_max = self.config.max_delay_ms;
            
            // Spawn a task
            let task = tokio::spawn(async move {
                // Release permit when dropped
                let _permit = permit;
                
                // Random delay before request - Scope rng 
                let delay = {
                    let mut rng = rand::rng();
                    rng.random_range(delay_min..=delay_max)
                };
                
                sleep(Duration::from_millis(delay)).await;
                
                println!("[{}] Extraindo: {}", phone_id, model);
                
                let mut detail = PhoneDetails::new(&Phone {
                    id: phone_id,
                    model: model.clone(),
                    url: url.clone(),
                    status: "processing".into()
                });
                
                match client.get(&url).header("User-Agent", ua).send().await {
                    Ok(resp) => {
                        detail.status_code = resp.status().as_u16();
                        if resp.status().is_success() {
                            if let Ok(html) = resp.text().await {
                                extract_specs(&html, &mut detail);
                            }
                        } else {
                            detail.error_message = Some(format!("HTTP {}", resp.status()));
                        }
                    },
                    Err(e) => {
                        detail.error_message = Some(e.to_string());
                    }
                }
                
                detail
            });
            
            tasks.push(task);
        }
        
        // Await all results
        for task in tasks {
            if let Ok(result) = task.await {
                details.push(result);
            }
        }
        
        Ok(details)
    }
}

// Logic to extract specs specifically from the table
fn extract_specs(html: &str, detail: &mut PhoneDetails) {
    let document = Document::from(html);
    
    // GSMArena structure: <div id="specs-list"> <table> ... </table> </div>
    let specs_list = Name("div").and(Attr("id", "specs-list"));
    
    let mut current_category = String::new();
    
    for tr in document.find(specs_list.descendant(Name("tr"))) {
        // Check for category header (th)
        if let Some(th) = tr.find(Name("th")).next() {
            current_category = th.text().to_lowercase();
        }
        
        // Find label and value
        let label_node = tr.find(Class("ttl")).next();
        let value_node = tr.find(Class("nfo")).next();
        
        if let (Some(l_node), Some(v_node)) = (label_node, value_node) {
            let label = l_node.text().to_lowercase();
            let value = v_node.text().trim().to_string();
            
            // Extract based on category AND label
            match current_category.as_str() {
                c if c.contains("body") => {
                    if label.contains("dimensions") {
                        detail.body_dimensions = Some(value);
                    } else if label.contains("weight") {
                        detail.body_weight = Some(value);
                    } else if label.contains("build") {
                        detail.body_build = Some(value);
                    } else if label.contains("sim") {
                        detail.sim = Some(value);
                    }
                },
                c if c.contains("display") => {
                    if label.contains("type") {
                        detail.display_type = Some(value);
                    } else if label.contains("size") {
                        detail.display_size = Some(value);
                    } else if label.contains("resolution") {
                        detail.resolution = Some(value);
                    } else if label.contains("protection") {
                        detail.protection = Some(value);
                    }
                },
                _ => {}
            }
        }
    }
}