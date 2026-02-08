mod config;
mod models;
mod scraper;
mod utils;

use crate::config::Config;
use crate::scraper::PhoneScraper;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("📱 GSMArena Scraper - High Speed Edition");
    println!("========================================\n");
    
    let config = Config::new();
    // Customize config if needed based on args or interactive mode, 
    // but for now default is fine (2 concurrent, 2-5s delay)
    
    let scraper = PhoneScraper::new(config)?;
    
    loop {
        println!("\nSelecione a operação:");
        println!("1. Coletar URLs (Realme - Exemplo)");
        println!("2. Coletar URLs (Personalizado)");
        println!("3. Extrair detalhes (Lote)");
        println!("4. Sair");
        print!("> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "1" => {
                let url = "https://www.gsmarena.com/realme-phones-118.php";
                let phones = scraper.scrape_phone_urls(url, 6).await?;
                utils::save_phones_to_csv(&phones, "realme_smartphones_recentes.csv")?;
                println!("✅ {} URLs coletadas.", phones.len());
            }
            "2" => {
                print!("Digite a URL da marca (ex: https://www.gsmarena.com/samsung-phones-9.php): ");
                io::stdout().flush()?;
                let mut url = String::new();
                io::stdin().read_line(&mut url)?;
                let url = url.trim();
                
                if url.is_empty() {
                    println!("❌ URL inválida");
                    continue;
                }
                
                let phones = scraper.scrape_phone_urls(url, 5).await?;
                utils::save_phones_to_csv(&phones, "smartphones_coletados.csv")?;
                println!("✅ {} URLs coletadas.", phones.len());
            }
            "3" => {
                print!("Arquivo CSV para carregar (Enter para 'realme_smartphones_recentes.csv'): ");
                io::stdout().flush()?;
                let mut filename = String::new();
                io::stdin().read_line(&mut filename)?;
                let filename = filename.trim();
                let filename = if filename.is_empty() { "realme_smartphones_recentes.csv" } else { filename };
                
                match utils::load_phones_from_csv(filename) {
                    Ok(phones) => {
                        println!("📄 {} telefones carregados.", phones.len());
                        let details = scraper.scrape_phone_details(&phones).await?;
                        
                        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M");
                        let csv_file = format!("results_{}.csv", timestamp);
                        let txt_file = format!("results_{}.txt", timestamp);
                        
                        utils::save_details_to_csv(&details, &csv_file)?;
                        utils::save_details_to_txt(&details, &txt_file)?;
                    },
                    Err(e) => println!("❌ Erro ao carregar arquivo: {}", e),
                }
            }
            "4" => {
                println!("👋 Saindo...");
                break;
            }
            _ => println!("❌ Opção inválida!"),
        }
    }
    
    Ok(())
}