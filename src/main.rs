

use indicatif::ProgressBar;
mod osufile_reader;
use osufile_reader::read_osu_file;
mod beatmap_stat_processor;
use beatmap_stat_processor::process_stats;
mod savedata_manager;
use savedata_manager::calculate_md5;
use jwalk::WalkDir as wd;
use rayon::prelude::*;
use rosu_pp::{Beatmap, BeatmapExt,GameMode};
use std::io::Write;
use std::{collections::HashSet, io};
use std::path::PathBuf;
use std::time::Instant;
mod full_data_struct;
use full_data_struct::{FullData, FullDataEnum};
use crate::savedata_manager::{read_main_data_csv, data_save_manager};

mod config_manager;
use config_manager::init_config;

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
fn process_beatmaps(songs_path:String,already_processed: &HashSet<String>,old_data: &[FullData], columns_config_vec: &[FullDataEnum]){
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
                                    avg_jump_distance: song.avg_jump_distance,
                                    avg_jump_speed: song.avg_jump_speed,
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
        println!("Processed {} Osu! Standard beatmaps in: {:?}\t({} beatmaps per second)",new_data.len(), duration,new_data.len() / duration.as_secs().max(1) as usize);

        let mut merged_data = Vec::new();
        merged_data.extend_from_slice(old_data);
        merged_data.extend(new_data);
        if let Ok(path) = data_save_manager(&merged_data,&columns_config_vec){
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
    let columns_config_vec = init_config("columns_config.txt".into());
    let mut old_data = Vec::new(); 
    if let Ok(data) = read_main_data_csv(){
        old_data=data;
    }
    let already_processed:HashSet<String> = old_data.iter().map(|item| item.md5.clone()).collect();
    print!("Enter Osu Song path (example D:\\osu\\Songs): ");
    io::stdout().flush().unwrap();
    let mut songs_path = String::new();
    io::stdin()
        .read_line(&mut songs_path)
        .expect("failed to read from stdin");
    process_beatmaps(songs_path,&already_processed,&old_data,&columns_config_vec);
    end();
}
fn end(){
    // !!! We make the user press enter to leave the program !!!
    let mut input = String::new();
    print!("Press enter to exit...");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).expect("Failed to read line");
}