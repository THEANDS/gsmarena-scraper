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
    
    writeln!(file, "SPECIFICAÇÕES DE TELA E CORPO")?;
    writeln!(file, "=========================================\n")?;
    
    let successful: Vec<&PhoneDetails> = details.iter()
        .filter(|d| d.has_specs())
        .collect();
    
    let failed: Vec<&PhoneDetails> = details.iter()
        .filter(|d| !d.has_specs())
        .collect();
    
    writeln!(file, "RESUMO:")?;
    writeln!(file, "  • Com dados: {}", successful.len())?;
    writeln!(file, "  • Sem dados: {}", failed.len())?;
    writeln!(file)?;
    
    for detail in details {
        writeln!(file, "ID: {}", detail.phone_id)?;
        writeln!(file, "Modelo: {}", detail.model)?;
        writeln!(file, "URL: {}", detail.url)?;
        
        writeln!(file, "\n[CORPO]")?;
        writeln!(file, "  Dimensões: {}", detail.body_dimensions.as_deref().unwrap_or("N/A"))?;
        writeln!(file, "  Peso: {}", detail.body_weight.as_deref().unwrap_or("N/A"))?;
        writeln!(file, "  Construção: {}", detail.body_build.as_deref().unwrap_or("N/A"))?;
        writeln!(file, "  SIM: {}", detail.sim.as_deref().unwrap_or("N/A"))?;
        
        writeln!(file, "\n[TELA]")?;
        writeln!(file, "  Tipo: {}", detail.display_type.as_deref().unwrap_or("N/A"))?;
        writeln!(file, "  Tamanho: {}", detail.display_size.as_deref().unwrap_or("N/A"))?;
        writeln!(file, "  Resolução: {}", detail.resolution.as_deref().unwrap_or("N/A"))?;
        writeln!(file, "  Proteção: {}", detail.protection.as_deref().unwrap_or("N/A"))?;
        
        if let Some(error) = &detail.error_message {
            writeln!(file, "\nErro: {}", error)?;
        }
        
        writeln!(file, "\n----------------------------------------\n")?;
    }
    
    println!("📝 Relatório salvo em: {}", filename);
    Ok(())
}