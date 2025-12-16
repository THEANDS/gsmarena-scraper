mod config;
mod models;
mod scraper;
mod utils;

use crate::config::Config;
use crate::scraper::PhoneScraper;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📱 GSM Arena Scraper - 1 por minuto");
    println!("===================================\n");
    
    let config = Config::new();
    let scraper = PhoneScraper::new(config)?;
    
    println!("Selecione a operação:");
    println!("1. Coletar URLs (9 páginas)");
    println!("2. Extrair detalhes (1 por minuto)");
    println!("3. Sair");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    match input.trim() {
        "1" => {
            let phones = scraper.scrape_phone_urls(Some(10))?;
            utils::save_phones_to_csv(&phones, "xiaomi_smartphones_recentes.csv")?;
            println!("✅ {} URLs coletadas.", phones.len());
        }
        "2" => {
            let phones = utils::load_phones_from_csv("xiaomi_smartphones_recentes.csv")?;
            println!("📄 {} telefones para processar", phones.len());
            
            // Estimar tempo total
            let total_minutes = phones.len();
            let total_hours = total_minutes as f32 / 60.0;
            println!("⏳ Tempo estimado: {} minutos ({:.1} horas)", total_minutes, total_hours);
            
            let details = scraper.scrape_one_per_minute(&phones)?;
            
            // Salvar resultados com timestamp
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M");
            let csv_file = format!("results_{}.csv", timestamp);
            let txt_file = format!("results_{}.txt", timestamp);
            
            utils::save_details_to_csv(&details, &csv_file)?;
            utils::save_details_to_txt(&details, &txt_file)?;
            
            // Exibir resumo
            display_summary(&details);
        }
        "3" => println!("👋 Saindo..."),
        _ => println!("❌ Opção inválida!"),
    }
    
    Ok(())
}

fn display_summary(details: &[crate::models::PhoneDetails]) {
    let successful = details.iter().filter(|d| d.has_display_info()).count();
    let failed = details.len() - successful;
    
    println!("\n📊 RESUMO:");
    println!("  ✅ Sucessos: {} ({:.1}%)", successful, 
             (successful as f32 / details.len() as f32) * 100.0);
    println!("  ❌ Falhas: {} ({:.1}%)", failed, 
             (failed as f32 / details.len() as f32) * 100.0);
}