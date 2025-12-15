mod config;
mod models;
mod scraper;
mod utils;

use crate::config::Config;
use crate::scraper::PhoneScraper;
use crate::utils::{load_phones_from_csv, save_details_to_csv, save_details_to_txt};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📱 GSM Arena Scraper - Sistema de Batching");
    println!("===========================================\n");
    
    // Configuração com batching
    let config = Config::new()
        .with_delay(2000)  // 2 segundos entre requisições
        .with_batch_size(20)
        .with_pause_settings(5, 120); // Pausa de 2min após 5 batches (100 reqs)
    
    let scraper = PhoneScraper::new(config)?;
    
    println!("Selecione a operação:");
    println!("1. Coletar URLs de smartphones (9 páginas)");
    println!("2. Extrair detalhes com batching (20 por lote)");
    println!("3. Extrair detalhes sem batching (apenas teste)");
    println!("4. Verificar progresso atual");
    println!("5. Limpar progresso e reiniciar");
    println!("6. Sair");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    match input.trim() {
        "1" => {
            let phones = scraper.scrape_phone_urls(Some(9))?;
            if let Err(e) = utils::save_phones_to_csv(&phones, "samsung_smartphones_recentes.csv") {
                eprintln!("❌ Erro ao salvar: {}", e);
            }
            println!("✅ {} URLs coletadas.", phones.len());
        }
        "2" => {
            match load_phones_from_csv("samsung_smartphones_recentes.csv") {
                Ok(phones) => {
                    println!("📄 {} telefones carregados", phones.len());
                    
                    if phones.is_empty() {
                        println!("❌ Nenhum telefone para processar!");
                        return Ok(());
                    }
                    
                    // Perguntar se quer continuar de onde parou
                    println!("Deseja continuar de onde parou? (s/n): ");
                    let mut resume_input = String::new();
                    std::io::stdin().read_line(&mut resume_input)?;
                    
                    let details = if resume_input.trim().to_lowercase() == "s" {
                        scraper.scrape_phone_details_with_batching(&phones)?
                    } else {
                        // Limpar progresso e começar do zero
                        if let Err(e) = std::fs::remove_file("scraper_progress.txt") {
                            if e.kind() != std::io::ErrorKind::NotFound {
                                eprintln!("⚠️  Não foi possível limpar progresso: {}", e);
                            }
                        }
                        scraper.scrape_phone_details_with_batching(&phones)?
                    };
                    
                    // Salvar resultados
                    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                    let csv_filename = format!("phone_details_{}.csv", timestamp);
                    let txt_filename = format!("phone_details_{}.txt", timestamp);
                    
                    if let Err(e) = save_details_to_csv(&details, &csv_filename) {
                        eprintln!("❌ Erro ao salvar CSV: {}", e);
                    }
                    if let Err(e) = save_details_to_txt(&details, &txt_filename) {
                        eprintln!("❌ Erro ao salvar TXT: {}", e);
                    }
                    
                    // Também salvar o mais recente sem timestamp
                    if let Err(e) = save_details_to_csv(&details, "phone_details_latest.csv") {
                        eprintln!("❌ Erro ao salvar latest CSV: {}", e);
                    }
                    
                    display_summary(&details);
                }
                Err(e) => eprintln!("❌ Erro ao carregar CSV: {}", e),
            }
        }
        "3" => {
            // Modo teste (sem batching)
            match load_phones_from_csv("samsung_smartphones_recentes.csv") {
                Ok(phones) => {
                    let test_phones = if phones.len() > 1 {
                        &phones[..1]
                    } else {
                        &phones
                    };
                    
                    println!("🧪 Modo teste: {} telefones", test_phones.len());
                    let details = scraper.scrape_phone_details(test_phones)?;
                    
                    if let Err(e) = save_details_to_csv(&details, "test_details.csv") {
                        eprintln!("❌ Erro: {}", e);
                    }
                    
                    display_summary(&details);
                }
                Err(e) => eprintln!("❌ Erro: {}", e),
            }
        }
        "4" => {
            check_progress();
        }
        "5" => {
            clear_progress();
        }
        "6" => println!("👋 Saindo..."),
        _ => println!("❌ Opção inválida!"),
    }
    
    Ok(())
}

fn check_progress() {
    let progress_file = "scraper_progress.txt";
    
    if std::path::Path::new(progress_file).exists() {
        if let Ok(content) = std::fs::read_to_string(progress_file) {
            let lines: Vec<&str> = content.trim().split('\n').collect();
            if lines.len() >= 3 {
                println!("\n📊 PROGRESSO ATUAL:");
                println!("  Batch atual: {}", lines[0]);
                println!("  Telefones processados: {}", lines[1]);
                println!("  Última atualização: {}", lines[2]);
            }
        } else {
            println!("❌ Não foi possível ler o arquivo de progresso.");
        }
    } else {
        println!("✅ Nenhum progresso salvo. Pronto para começar!");
    }
}

fn clear_progress() {
    let progress_file = "scraper_progress.txt";
    
    if std::path::Path::new(progress_file).exists() {
        if let Err(e) = std::fs::remove_file(progress_file) {
            eprintln!("❌ Erro ao limpar progresso: {}", e);
        } else {
            println!("✅ Progresso limpo com sucesso!");
        }
    } else {
        println!("ℹ️  Nenhum progresso para limpar.");
    }
}

fn display_summary(details: &[crate::models::PhoneDetails]) {
    let successful = details.iter().filter(|d| d.has_display_info()).count();
    let failed = details.len() - successful;
    
    println!("\n📊 RESUMO FINAL:");
    println!("  ✅ Sucessos: {} ({:.1}%)", successful, 
             (successful as f32 / details.len() as f32) * 100.0);
    println!("  ❌ Falhas: {} ({:.1}%)", failed, 
             (failed as f32 / details.len() as f32) * 100.0);
    
    if successful > 0 {
        println!("\n📈 EXEMPLOS DE DADOS EXTRAÍDOS:");
        for detail in details.iter().filter(|d| d.has_display_info()).take(5) {
            println!("  • {}: Ratio {}, Área {} cm², {}",
                     detail.model,
                     detail.display_ratio.as_deref().unwrap_or("N/A"),
                     detail.display_area_cm2.as_deref().unwrap_or("N/A"),
                     detail.resolution.as_deref().unwrap_or("N/A"));
        }
    }
}