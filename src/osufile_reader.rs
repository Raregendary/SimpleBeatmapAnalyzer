use std::fs;
use std::io::{BufRead, BufReader};
#[inline(always)]
pub fn read_osu_file(filename: &str) -> (String, String, String, String, String) {
    let file = fs::File::open(filename).expect("Failed to open file");
    let reader = BufReader::new(file);

    let mut title = String::new();
    let mut creator = String::new();
    let mut version = String::new();
    let mut beatmap_id = String::new();
    //let mut beatmap_set_id = String::new();
    let mut b1=true;
    let mut b2=false;
    let mut b3=false;
    let mut b4=false;
    let mut b5=false;
    
    for line in reader.lines() {
        if let Ok(line) = line {
            if b1 && line.starts_with("Tit"){
                b1=false;
                b2=true;
                title = line.split(":").nth(1).unwrap().to_string();
                continue;
            }
            if b2 && line.starts_with("Cr") {
                b2 = false;
                b3 = true;
                creator = line.split(":").nth(1).unwrap().to_string();
                continue;
            }
            if b3 && line.starts_with("Ver") {
                b3 = false;
                b4 = true;
                version = line.split(":").nth(1).unwrap().to_string();
                continue;
            }
            if b4 && line.starts_with("Beatm"){
                b4 = false;
                b5 = true;
                beatmap_id = line.split(":").nth(1).unwrap().to_string();
                continue;
            }
            if b4 && line.starts_with("Beatm"){//BeatmapSetID
                return (title,creator, version, beatmap_id, line.split(":").nth(1).unwrap().to_string());
            }
        }
        
    }
    (title,creator, version, beatmap_id, "".into())
}