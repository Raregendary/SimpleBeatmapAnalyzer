

use indicatif::ProgressBar;
mod osufile_reader;
use osufile_reader::read_osu_file;
mod beatmap_stat_processor;
use beatmap_stat_processor::process_stats;
mod savedata_manager;
use savedata_manager::{calculate_md5};
use jwalk::WalkDir as wd;
use rayon::prelude::*;
use rosu_pp::{Beatmap, BeatmapExt,GameMode};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Instant;
use std::io::{self, Write};
use serde::{Deserialize,Serialize};

use crate::savedata_manager::{read_main_data_csv, data_save_manager};

#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct FullData {
    #[serde(rename = "Title")]
    title: String,
    #[serde(rename = "DifName")]
    version: String,
    #[serde(rename = "MapID")]
    beatmap_id: String,
    #[serde(rename = "MapSetID")]
    beatmap_set_id: String,
    #[serde(rename = "CS")]
    cs: f32,
    #[serde(rename = "AR")]
    ar: f32,
    #[serde(rename = "OD")]
    od: f32,
    #[serde(rename = "HP")]
    hp: f32,
    #[serde(rename = "Stars")]
    stars_nm: f32,
    #[serde(rename = "DT_Stars")]
    stars_dt: f32,
    #[serde(rename = "HR_Stars")]
    stars_hr: f32,
    #[serde(rename = "NM_99")]
    nm: f32,
    #[serde(rename = "DT_99")]
    dt: f32,
    #[serde(rename = "HR_99")]
    hr: f32,
    #[serde(rename = "PlayableLength")]
    playable_length: i32,
    #[serde(rename = "BPM")]
    bpm: i32,           // most common bpm
    #[serde(rename = "Doubles")]
    doubles: f32,
    #[serde(rename = "Triples")]
    triples: f32,
    #[serde(rename = "Bursts")]
    bursts: f32,        // 3-12     1/4th
    #[serde(rename = "Streams")]
    streams: f32,       // 13-32    1/4th
    #[serde(rename = "DeathStreams")]
    deathstreams: f32,  // 33+      1/4th
    #[serde(rename = "ShortJumps")]
    short_jumps: f32,   // 3-12     1/2th
    #[serde(rename = "MidJumps")]
    mid_jumps: f32,     // 13-32    1/2th
    #[serde(rename = "Longjumps")]
    long_jumps: f32,    // 33+      1/2th
    #[serde(rename = "Quads")]
    quads: f32,
    #[serde(rename = "FCDBI")]
    fcdbi: f32,//FINGER CONTROL DOUBLE BURSTS INDEX
    #[serde(rename = "SI")]
    si: f32,//Stream index
    #[serde(rename = "JI")]
    ji: f32,//Jump index
    #[serde(rename = "LongestStream")]
    longest_stream: u32,
    #[serde(rename = "Streams100")]
    streams100: u32, // how much 100 or higher note streams are in the map as counter
    #[serde(rename = "MD5")]
    md5: String,
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

#[inline(always)]
fn process_beatmaps(songs_path:String,already_processed: &HashSet<String>,old_data: &[FullData]){
    //Try to get all the .osu files
    if let Ok(files) = find_osu_files6(songs_path.trim().into()){
        println!("Found {} osu beatmaps.\nProcessing Osu! Standard beatmaps:",files.len());
        let start = Instant::now();
        let bar = ProgressBar::new(files.len() as u64);
        //process all the beatmaps in parallel if they are from Standard and save them in output vector.
        let new_data: Vec<FullData> = files.par_iter()
            .fold(|| Vec::with_capacity(files.len()), |mut acc, path| {
                bar.inc(1);
                let md5 = calculate_md5(path);
                if !already_processed.contains(&md5) {
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
                                    longest_stream: song.longest_stream,
                                    streams100: song.streams100,
                                    md5: md5,
                                });
        
                            }
                            
                        }
                    }
                }
                acc  
            })
            .flatten()
            .collect();
        bar.finish();
        let duration = start.elapsed();
        println!("Processed {} Osu! Standard beatmaps in: {:?}\t({} beatmaps per second)",new_data.len(), duration,new_data.len() / duration.as_secs() as usize);

        if let Ok(path) = data_save_manager(&new_data,&old_data){
            println!("CSV file saved successfuly ->\t{}",path.to_str().unwrap());
        } else {
            println!("FAILED TO SAVE CSV FILE!!!");
        }
    }
    else {
        println!("Wrong path");
    }
}
#[inline(always)]
fn test_md5speed(songs_path:String){
    //Try to get all the .osu files
    if let Ok(files) = find_osu_files6(songs_path.trim().into()){
        println!("Found {} osu beatmaps.\nProcessing Osu! Standard beatmaps:",files.len());
        let start = Instant::now();
        let bar = ProgressBar::new(files.len() as u64);
        //process all the beatmaps in parallel if they are from Standard and save them in output vector.
        let datas: Vec<String> = files.par_iter()
            .fold(|| Vec::with_capacity(files.len()), |mut acc, path| {
                bar.inc(1);
                acc.push(calculate_md5(path));
                acc
            })
            .flatten()
            .collect();
        bar.finish();
        let duration = start.elapsed();
        println!("Processed {} Osu! Standard beatmaps in: {:?}\t({} beatmaps per second)",datas.len(), duration,datas.len() / duration.as_millis() as usize);
        
    }
}
fn main() {
    let mut old_data = Vec::new(); 
    if let Ok(data) = read_main_data_csv(){
        old_data=data;
    }
    
    let already_processed:HashSet<String> = old_data.iter().map(|item| item.md5.clone()).collect();
    println!("{:?}",already_processed);
    println!("{:?}",old_data.len());
    print!("Enter Osu Song path (example D:\\osu\\Songs): ");
    io::stdout().flush().unwrap();
    let mut songs_path = String::new();
    io::stdin()
        .read_line(&mut songs_path)
        .expect("failed to read from stdin");
    process_beatmaps(songs_path,&already_processed,&old_data);
    end();
}
fn end(){
    // !!! We make the user press enter to leave the program !!!
    let mut input = String::new();
    print!("Press enter to exit...");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).expect("Failed to read line");
}