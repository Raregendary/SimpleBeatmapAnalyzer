
use std::fs::{File, self};
use std::io::{BufReader, Read, self};
use std::env;
use std::error::Error;
use md5::{Md5, Digest};
use csv::WriterBuilder;
use std::io::BufWriter;
use std::path::{PathBuf, Path};
use crate::FullData;
use crate::full_data_struct::{FullDataEnum, FullDataTrait};

#[inline(always)]
pub fn data_save_manager(full_data: &[FullData],columns_config_vec: &[FullDataEnum]) -> Result<PathBuf, Box<dyn Error>>{
    write_main_data_csv_new(full_data,columns_config_vec)
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
#[allow(dead_code)]
#[inline(always)]
pub fn calculate_md5(file_path: &str) -> String {
    let f = File::open(file_path).unwrap();
    let mut reader = BufReader::new(f);
    let mut hasher = Md5::new();
    let mut buffer = [0; 1024];
    loop {
        let count = reader.read(&mut buffer).unwrap();
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
fn write_main_data_csv_new(full_data: &[FullData],columns_config_vec: &[FullDataEnum]) -> Result<PathBuf, Box<dyn Error>> {
    create_data_dir()?;
    let mut file_path = env::current_dir()?;
    file_path.push("data\\data.csv");
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
    
    //for now we do a copy later will be much more
    let src_file = Path::new("data\\data.csv");
    let dest_file = Path::new("results.csv");
    fs::copy(src_file, dest_file)?;
    //return that full file path
    Ok(file_path)
}
