
use std::collections::HashMap;
use std::fs::{File, self};
use std::hash::Hash;
use std::io::{BufReader, Read, self};
use std::env;
use std::error::Error;
use md5::{Md5, Digest};
use csv::WriterBuilder;
use rosu_pp::GameMode;
use strum::IntoEnumIterator;
use std::io::BufWriter;
use std::path::{PathBuf, Path};
use crate::FullData;
use crate::full_data_struct::{FullDataEnum, FullDataTrait, SavableFullData, SavableFullDataTrait, GameMode_Serialized};

#[inline(always)]
fn copy_options_config(options_config_path: String) -> Result<(), Box<dyn Error>> {
    let from = &options_config_path;
    let to = format!("data\\{}", &options_config_path);
    fs::copy(from, to)?;
    Ok(())
}
#[inline(always)]
fn create_data_dir() -> io::Result<()> {
    let mut dir_path = std::env::current_dir()?;
    dir_path.push("data");
    fs::create_dir_all(&dir_path)?;
    Ok(())
}
#[inline(always)]
pub fn calculate_md5(file_path: &str) -> String {
    let f = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => return String::new(),
    };
    let mut reader = BufReader::new(f);
    let mut hasher = Md5::new();
    let mut buffer = [0; 1024];
    loop {
        let count = match reader.read(&mut buffer) {
            Ok(count) => count,
            Err(_) => return String::new(),
        };
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let result = hasher.finalize();
    format!("{:x}", result)
}
//rewrite starts here
#[inline(always)]
pub fn data_save_manager(full_data: &HashMap<String, SavableFullData>,columns_config_vec: &[FullDataEnum],options_config_path: String) -> Result<PathBuf, Box<dyn Error>>{
    if let Ok(data_path) = save_bin_data(full_data) {
        copy_options_config(options_config_path).unwrap_or_else(|_| {
            println!("Failed to save options config!!!");
        });
        if let Ok(results_path) = write_results_csv(full_data,columns_config_vec,"results.csv") {
            return Ok(results_path);
        }
    }
    Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Failed to save data!!!")))
}
#[inline(always)]
fn write_results_csv(full_data: &HashMap<String, SavableFullData>,columns_config_vec: &[FullDataEnum],results_data_path:&str) -> Result<PathBuf, Box<dyn Error>> {
    create_data_dir()?;
    let mut file_path = env::current_dir()?;
    file_path.push(results_data_path);
    let file = File::create(&file_path)?;
    let mut writer = csv::Writer::from_writer(BufWriter::new(file));
    // Write the header row
    let mut headers = Vec::new();
    for column in columns_config_vec.iter() {
        headers.push(format!("{:?}", column));
    }
    writer.write_record(&headers)?;

    for data in full_data {
        if data.1.beatmap.mode == GameMode_Serialized::Osu {
            writer.write_record(&columns_config_vec.iter().map(|column| data.1.get_string(column)).collect::<Vec<String>>())?;
        }
    }
    
    Ok(file_path)
}
#[inline(always)]
fn save_bin_data(data: &HashMap<String, SavableFullData>) -> std::io::Result<()> {
    let file = File::create("data\\data_v0-9-3.bin")?;
    let writer = BufWriter::new(file);
    bincode::serialize_into(writer, data).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(())
}
#[inline(always)]
pub fn load_bin_data() -> std::io::Result<HashMap<String, SavableFullData>> {
    let file = File::open("data\\data_v0-9-3.bin")?;
    let reader = BufReader::new(file);
    let data: HashMap<String, SavableFullData> = bincode::deserialize_from(reader).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(data)
}