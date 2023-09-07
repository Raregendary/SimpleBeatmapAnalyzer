use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use indicatif::ProgressBar;
mod osufile_reader;
use osufile_reader::read_osu_file;
mod beatmap_stat_processor;
use beatmap_stat_processor::process_stats;
use jwalk::WalkDir as wd;
use rayon::prelude::*;
use rosu_pp::{Beatmap, BeatmapExt,GameMode};

use std::env;
use std::error::Error;
use std::time::Instant;
use md5::{Md5, Digest};
use std::io::{self, Write};
use csv::Writer;
use std::io::BufWriter;
#[derive(Debug,Clone)]
pub struct FullData {
    title: String,
    version: String,
    beatmap_id: String,
    beatmap_set_id: String,
    cs: f32,
    ar: f32,
    od: f32,
    hp: f32,
    stars_nm: f32,
    stars_dt: f32,
    stars_hr: f32,
    nm: f32,
    dt: f32,
    hr: f32,
    playable_length: i32,
    bpm: i32,           // most common bpm
    doubles: f32,
    triples: f32,
    bursts: f32,        // 3-12     1/4th
    streams: f32,       // 13-32    1/4th
    deathstreams: f32,  // 33+      1/4th
    short_jumps: f32,   // 3-12     1/2th
    mid_jumps: f32,     // 13-32    1/2th
    long_jumps: f32,    // 33+      1/2th
    quads: f32,
    fcdbi: f32,//FINGER CONTROL DOUBLE BURSTS INDEX
    si: f32,//Stream index
    ji: f32,//Jump index
}
#[inline(always)]
fn find_osu_files6(path: String) -> Result<Vec<String>, io::Error> {
    println!("Searching for \".osu\" files, please be patient...");
    let mut files: Vec<String> = Vec::with_capacity(65_536);
    let path = if path == "69" {
        PathBuf::from("D:\\osu\\Songs")
    } else {
        PathBuf::from(path)
    };
    for entry in wd::new(path).into_iter() {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_file() && entry.path().extension().unwrap_or_default() == "osu" {
                    files.push(entry.path().to_string_lossy().into_owned());
                }
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(files)
}
#[allow(dead_code)]
#[inline(always)]
fn calculate_md5(file_path: &str) -> String {
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
#[inline(always)]
fn process_beatmaps(songs_path:String){
    //Try to get all the .osu files
    if let Ok(files) = find_osu_files6(songs_path.trim().into()){
        println!("Found {} osu beatmaps.\nProcessing Osu! Standard beatmaps:",files.len());
        let start = Instant::now();
        let bar = ProgressBar::new(files.len() as u64);
        //process all the beatmaps in parallel if they are from Standard and save them in output vector.
        let datas: Vec<FullData> = files.par_iter()
            .fold(|| Vec::with_capacity(files.len()), |mut acc, path| {
                bar.inc(1);
                if let Ok(map) = Beatmap::from_path(path) {
                    if map.mode == GameMode::Osu {
                        if let Ok(song) = process_stats(&map.timing_points,&map.hit_objects,map.cs){
                            //using my function to parse title version map id and map set id because rosu pp doesnt have -_-
                            let (title1, version1, beatmap_id1, beatmap_set_id1) = read_osu_file(path);
                            let nm = map.pp().accuracy(99.0).calculate();
                            let dt = map.pp().mods(64).accuracy(99.0).calculate();
                            let hr = map.pp().mods(16).accuracy(99.0).calculate();
                            acc.push(FullData{
                                title: title1,
                                version: version1,
                                beatmap_id: beatmap_id1,
                                beatmap_set_id: beatmap_set_id1,
                                cs: map.cs,
                                ar: map.ar,
                                od: map.od,
                                hp: map.hp,
                                stars_nm: nm.stars() as f32, 
                                stars_dt: dt.stars() as f32, 
                                stars_hr: hr.stars() as f32, 
                                nm: nm.pp() as f32, 
                                dt: dt.pp() as f32, 
                                hr: hr.pp() as f32, 
                                playable_length: song.playable_length,
                                bpm: song.bpm,
                                doubles: song.doubles,
                                triples: song.triples,
                                bursts: song.bursts,
                                streams: song.streams,
                                deathstreams: song.deathstreams,
                                short_jumps: song.short_jumps,
                                mid_jumps: song.mid_jumps,
                                long_jumps: song.long_jumps,
                                quads: song.quads,
                                fcdbi: song.fcdbi,
                                si: song.si,
                                ji: song.ji,
                            });
    
                        }
                        
                    }
                }
                acc
            })
            .flatten()
            .collect();
        bar.finish();
        let duration = start.elapsed();
        println!("Processed {} Osu! Standard beatmaps in: {:?}\t({} beatmaps per second)",datas.len(), duration,datas.len() / duration.as_secs() as usize);

        if let Ok(path) = write_to_csv(datas){
            println!("CSV file saved successfuly ->\t{}",path.to_str().unwrap());
        } else {
            println!("FAILED TO SAVE CSV FILE!!!");
        }
    }
    else {
        println!("Wrong path");
    }
}
fn main() {
    print!("Enter Osu Song path (example D:\\osu\\Songs): ");
    io::stdout().flush().unwrap();
    let mut songs_path = String::new();
    io::stdin()
        .read_line(&mut songs_path)
        .expect("failed to read from stdin");
    process_beatmaps(songs_path);
    end();
}

#[inline(always)]
fn write_to_csv(combined_vec: Vec<FullData>) -> Result<PathBuf, Box<dyn Error>> {
    let mut file_path = env::current_dir()?;
    file_path.push("results.csv");
    let file = File::create(&file_path)?;
    let mut writer = Writer::from_writer(BufWriter::new(file));

    // Write the header row
    writer.write_record(&[
        "Title",
        "DifName",
        "MapID",
        "Stars",
        "BPM",
        "Bursts",
        "Streams",
        "DeathStreams",
        "ShortJumps",
        "MidJumps",
        "Longjumps",
        "Doubles",
        "Triples",
        "SI",//Stream index
        "JI",//Jumps indedx
        "FCDBI",//FINGER CONTROLL DOUBLE BURSTS INDEX 
        "PlayableLength",
        "CS",
        "AR",
        "OD",
        "HP",
        "NM_99",
        "DT_99",
        "HR_99",
        "DT_Stars",
        "HR_Stars",
        "DT_BPM",
        "DT_AR",
        "HR_AR",
        "HR_CS",
        "Quads",
        "MapSetID"
    ])?;

    // Write the data rows
    for combined in combined_vec {
        writer.write_record(&[
            &combined.title,
            &combined.version,
            &combined.beatmap_id.to_string(),
            &format!("{:.2}", combined.stars_nm),
            &combined.bpm.to_string(),
            &format!("{:.2}", combined.bursts),
            &format!("{:.2}", combined.streams),
            &format!("{:.2}", combined.deathstreams),
            &format!("{:.2}", combined.short_jumps),
            &format!("{:.2}", combined.mid_jumps),
            &format!("{:.2}", combined.long_jumps),
            &format!("{:.2}", combined.doubles),
            &format!("{:.2}", combined.triples),
            &format!("{:.2}", combined.si),
            &format!("{:.2}", combined.ji),
            &format!("{:.2}", combined.fcdbi),
            &combined.playable_length.to_string(),
            &combined.cs.to_string(),
            &combined.ar.to_string(),
            &combined.od.to_string(),
            &combined.hp.to_string(),
            &format!("{:.2}", combined.nm),
            &format!("{:.2}", combined.dt),
            &format!("{:.2}", combined.hr),
            &format!("{:.2}", combined.stars_dt),
            &format!("{:.2}", combined.stars_hr),
            &((combined.bpm as f32 * 1.5).ceil() as i32).to_string(),
            &format!("{:.2}", apply_dt_to_ar(combined.ar)),
            &format!("{:.2}", (combined.ar * 1.4).min(10.0)),
            &format!("{:.2}", (combined.cs * 1.3).min(10.0)),
            &format!("{:.2}", combined.quads),
            &combined.beatmap_set_id.to_string()
        ])?;
    }
    //return that full file path
    Ok(file_path)
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
fn end(){
    // !!! We make the user press enter to leave the program !!!
    let mut input = String::new();
    print!("Press enter to exit...");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).expect("Failed to read line");
}