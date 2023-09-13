use std::{io::{Write, BufWriter}, fs::{self, File}, str::FromStr};
use crate::full_data_struct::FullDataEnum;
use std::path::Path;
use strum::IntoEnumIterator;
#[inline(always)]
pub fn init_options_config(path: String)  -> OptionsConfig {
//
    let path: &Path = Path::new(&path);
    if !path.exists() {
        if let Ok(_) = write_options_config_file(path) {
            println!("Successfully created options config file! (loading defaults)");
            println!("To change stream spacing or jumps spacing or default osu path and others you can edit options_config.txt file");
            println!("Note that changing anything in it will require full data recalculation!!!");
        }
        else{
            println!("Failed to create optionsconfig file (loading defaults), please restart the program if you want your values to be used!!!");
            return get_default_options();
        }
    }
    read_config(&path)
}

#[inline(always)]
fn read_config(path: &Path) -> OptionsConfig {
    let contents = fs::read_to_string(&path);
    match contents {
        Ok(data) => {
            let mut file_path = String::new();
            let mut stream_distance = 16.0;
            let mut jump_distance = 110.0;

            for line in data.lines() {
                if !line.starts_with("/") {
                    let parts: Vec<&str> = line.split("=").collect();
                    if parts.len() == 2 {
                        match parts[0].trim() {
                            "FilePath" => file_path = parts[1].trim().to_string(),
                            "StreamDistance" => stream_distance = parts[1].trim().parse::<f32>().unwrap_or(16.0),
                            "JumpDistance" => jump_distance = parts[1].trim().parse::<f32>().unwrap_or(110.0),
                            _ => continue,
                        }
                    }
                }
            }

            OptionsConfig {
                file_path,
                min_stream_distance: stream_distance,
                min_jump_distance: jump_distance,
            }
        },
        Err(_) => get_default_options()
    }
}

#[inline(always)]
fn write_options_config_file(path: &Path) -> std::io::Result<()> {
    let file = File::create(&path)?;
    let mut writer = BufWriter::new(file);
    writeln!(writer, "//---CHANGING ANYTHING IN THIS FILE WILL RESULT IN A FULL DATA RECALCULATION!!!---")?;
    writeln!(writer, "//You can change the value used to calculate streams which by default is less than 16 pixels edge to edge.")?;
    writeln!(writer, "//You can change the value used to calculate jumps which by default is greater than 110 pixels edge to edge.")?;
    writeln!(writer, "Defaults: [FilePath=\"\", StreamDistance=16, JumpDistance=110]  (entering wrong value or type of value will load a default instead)")?;
    writeln!(writer, "FilePath=")?;
    writeln!(writer, "StreamDistance=16.0")?;
    writeln!(writer, "JumpDistance=110.0")?;
    Ok(())
}
#[inline(always)]
pub fn get_default_options() -> OptionsConfig {
    OptionsConfig {
        file_path: String::new(),
        min_stream_distance: 16.0,
        min_jump_distance: 110.0,
    }
}
pub struct OptionsConfig {
    pub file_path: String,
    pub min_stream_distance: f32,
    pub min_jump_distance: f32,
}
