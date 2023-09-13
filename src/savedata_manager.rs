
use std::fs::{File, self};
use std::io::{BufReader, Read, self};
use std::env;
use std::error::Error;
use md5::{Md5, Digest};
use csv::WriterBuilder;
use strum::IntoEnumIterator;
use std::io::BufWriter;
use std::path::{PathBuf, Path};
use crate::FullData;
use crate::full_data_struct::{FullDataEnum, FullDataTrait};

#[inline(always)]
pub fn data_save_manager(full_data: &[FullData],columns_config_vec: &[FullDataEnum],options_config_path: String) -> Result<PathBuf, Box<dyn Error>>{
    if let Ok(data_path) = write_main_data_csv(full_data,"data\\data.csv") {
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
fn copy_options_config(options_config_path: String) -> Result<(), Box<dyn Error>> {
    let from = &options_config_path;
    let to = format!("data\\{}", &options_config_path);
    fs::copy(from, to)?;
    Ok(())
}
#[inline(always)]
pub fn read_main_data_csv() -> Result<Vec<FullData>, Box<dyn Error>> {
    let file = File::open("data\\data.csv")?;
    let mut reader = csv::Reader::from_reader(BufReader::new(file));

    let mut data = Vec::new();
    
    for result in reader.deserialize() {
        let record: FullData = result?;
        data.push(record);
    }
    
    Ok(data)
}
#[inline(always)]
fn create_data_dir() -> io::Result<()> {
    let mut dir_path = std::env::current_dir()?;
    dir_path.push("data");
    fs::create_dir_all(&dir_path)?;
    Ok(())
}

#[inline(always)]
fn apply_dt_to_ar(original_ar: f32) -> f32 {
    //calculating the ar change from DT mode
    let ms = if original_ar > 5.0 {
        200.0 + (11.0 - original_ar) * 100.0
    } else {
        800.0 + (5.0 - original_ar) * 80.0
    };
    let new_ar = if ms < 300.0 {
        11.0
    } else if ms < 1200.0 {
        ((11.0 - (ms - 300.0) / 150.0) * 100.0).round() / 100.0
    } else {
        ((5.0 - (ms - 1200.0) / 120.0) * 100.0).round() / 100.0
    };
    new_ar
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
pub fn write_serde_to_csv(data: &[FullData]) -> Result<(), Box<dyn Error>> {
    let mut writer = WriterBuilder::new().from_path("data/test.csv")?;
    writer.serialize(data)?;
    writer.flush()?;
    Ok(())
}
#[inline(always)]
fn write_main_data_csv(full_data: &[FullData],main_data_path:&str) -> Result<PathBuf, Box<dyn Error>> {
    create_data_dir()?;
    let mut file_path = env::current_dir()?;
    file_path.push(main_data_path);
    let file = File::create(&file_path)?;
    let mut writer = csv::Writer::from_writer(BufWriter::new(file));
    // Write the header row
    let mut headers = Vec::new();
    for column in FullDataEnum::iter() {
        headers.push(format!("{:?}", column));
    }
    writer.write_record(&headers)?;

    for data in full_data {
        writer.write_record(&FullDataEnum::iter().map(|column| data.get_string(&column)).collect::<Vec<String>>())?;
    }
    
    //return that full file path
    Ok(file_path)
}

#[inline(always)]
fn write_results_csv(full_data: &[FullData],columns_config_vec: &[FullDataEnum],results_data_path:&str) -> Result<PathBuf, Box<dyn Error>> {
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
        writer.write_record(&columns_config_vec.iter().map(|column| data.get_string(column)).collect::<Vec<String>>())?;
    }
    
    Ok(file_path)
}
