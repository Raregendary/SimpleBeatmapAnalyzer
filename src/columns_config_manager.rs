use std::{io::{Write, BufWriter}, fs::{self, File}, str::FromStr};
use crate::full_data_struct::FullDataEnum;
use std::path::Path;
use strum::IntoEnumIterator;
#[inline(always)]
pub fn init_columns_config(path: String)  -> Vec<FullDataEnum> {
//
    let path: &Path = Path::new(&path);
    if !path.exists() {
        if let Ok(_) = write_enum_to_file(path) {
            println!("Successfully created columns config file! (loading defaults)");
            println!("To change the order or hide the columns, edit the file column_config.txt and restart the program")
        }
        else{
            println!("Failed to create columns config file (loading defaults), please restart the program.");
            return get_all_enum_values();
        }
    }
    read_config(&path)
}

#[inline(always)]
fn read_config(path: &Path) -> Vec<FullDataEnum> {
    let contents = fs::read_to_string(&path).expect("Something went wrong reading the file column_config.txt file");
    let mut my_vec: Vec<FullDataEnum> = Vec::new();
    for line in contents.lines() {
        if !line.starts_with("/") {
            let parts: Vec<&str> = line.split("=").collect();
            if parts.len() == 2 && parts[1].trim() == "1" {
                match FullDataEnum::from_str(parts[0].trim()) {
                    Ok(value) => my_vec.push(value),
                    Err(_) => continue,
                }
            }
        }
    }
    my_vec.push(FullDataEnum::MD5);
    my_vec

}

#[inline(always)]
fn write_enum_to_file(path: &Path) -> std::io::Result<()> {
    let file = File::create(&path)?;
    let mut writer = BufWriter::new(file);
    writeln!(writer, "//In this file you can select which columns to be displayed in results.csv")?;
    writeln!(writer, "//You can swap thier order, to disable one simply replace the 1 with 0")?;
    writeln!(writer, "//MD5 is always included for unique identifier so already computed maps wont be computed again.")?;
    writeln!(writer, "Defaults: {:?}", get_all_enum_values())?;
    let variants: Vec<_> = FullDataEnum::iter().collect();
    for variant in &variants[0..variants.len()-1] {
        writeln!(writer, "{}=1", format!("{:?}", variant))?;
    }
    Ok(())
}
#[inline(always)]
pub fn get_all_enum_values() -> Vec<FullDataEnum> {
    FullDataEnum::iter().collect()
}