use std::fs::File;
use std::io::{Write, BufReader, BufWriter};
use std::path::Path;
use csv::{ReaderBuilder, WriterBuilder};
use crate::models::{Phone, PhoneDetails};




pub fn save_phones_to_csv(phones: &[Phone], filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(filename)?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(BufWriter::new(file));
    
    for phone in phones {
        wtr.serialize(phone)?;
    }
    
    wtr.flush()?;
    println!("✅ Telefones salvos em: {}", filename);
    Ok(())
}

pub fn load_phones_from_csv(filename: &str) -> Result<Vec<Phone>, Box<dyn std::error::Error>> {
    if !Path::new(filename).exists() {
        return Err(format!("Arquivo {} não encontrado", filename).into());
    }
    
    let file = File::open(filename)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(BufReader::new(file));
    
    let mut phones = Vec::new();
    
    for result in rdr.deserialize() {
        let phone: Phone = result?;
        phones.push(phone);
    }
    
    Ok(phones)
}

pub fn save_details_to_csv(details: &[PhoneDetails], filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(filename)?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(BufWriter::new(file));
    
    for detail in details {
        wtr.serialize(detail)?;
    }
    
    wtr.flush()?;
    println!("✅ Detalhes salvos em: {}", filename);
    Ok(())
}

pub fn save_details_to_txt(details: &[PhoneDetails], filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(filename)?;
    
    writeln!(file, "DETALHES DE DISPLAY - SMARTPHONES SAMSUNG")?;
    writeln!(file, "=========================================\n")?;
    
    let successful: Vec<&PhoneDetails> = details.iter()
        .filter(|d| d.has_display_info())
        .collect();
    
    let failed: Vec<&PhoneDetails> = details.iter()
        .filter(|d| !d.has_display_info())
        .collect();
    
    writeln!(file, "RESUMO:")?;
    writeln!(file, "  • Completos: {}", successful.len())?;
    writeln!(file, "  • Falhos: {}", failed.len())?;
    writeln!(file)?;
    
    for detail in details {
        writeln!(file, "ID: {}", detail.phone_id)?;
        writeln!(file, "Modelo: {}", detail.model)?;
        writeln!(file, "Ratio: {}", detail.display_ratio.as_deref().unwrap_or("N/A"))?;
        writeln!(file, "Área: {} cm²", detail.display_area_cm2.as_deref().unwrap_or("N/A"))?;
        writeln!(file, "Resolução: {}", detail.resolution.as_deref().unwrap_or("N/A"))?;
        writeln!(file, "Tamanho: {}", detail.screen_size.as_deref().unwrap_or("N/A"))?;
        writeln!(file, "PPI: {}", detail.ppi.as_deref().unwrap_or("N/A"))?;
        writeln!(file, "URL: {}", detail.url)?;
        writeln!(file, "Status HTTP: {}", detail.status_code)?;
        
        if let Some(error) = &detail.error_message {
            writeln!(file, "Erro: {}", error)?;
        }
        
        writeln!(file, "---")?;
    }
    
    println!("📝 Relatório salvo em: {}", filename);
    Ok(())
}