

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
use std::hash::Hash;
use std::io::Write;
use std::{collections::HashSet, io};
use std::path::{PathBuf, self};
use std::time::Instant;
mod full_data_struct;
use full_data_struct::{FullData, FullDataEnum};
use crate::savedata_manager::{read_main_data_csv, data_save_manager};
mod options_config_manager;
use options_config_manager::{init_options_config, OptionsConfig};
mod columns_config_manager;
use columns_config_manager::init_columns_config;

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
fn process_beatmaps(songs_path:String,already_processed: &HashSet<String>,old_data: &[FullData], columns_config_vec: &[FullDataEnum],options: OptionsConfig,are_options_equal: bool,options_config_path:String){
    //Try to get all the .osu files
    if let Ok(files) = find_osu_files6(songs_path.trim().into()){
        println!("Found {} osu beatmaps.\nProcessing Osu! Standard beatmaps:",files.len());
        let start = Instant::now();
        let new_data = data_processor(&files,already_processed,options);
        let duration = start.elapsed();
        println!("Processed {} Osu! Standard beatmaps in: {:?}\t({} beatmaps per second)",new_data.len(), duration,new_data.len() / duration.as_secs().max(1) as usize);
        let mut merged_data = Vec::new();
        if are_options_equal{
            merged_data.extend_from_slice(old_data);
            merged_data.extend(new_data);
        }
        else {
            merged_data.extend(new_data)
        }
        if let Ok(path) = data_save_manager(&merged_data,&columns_config_vec,options_config_path){
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
fn data_processor(files: &[String],already_processed: &HashSet<String>,options: OptionsConfig,) -> Vec<FullData>{
    let bar = ProgressBar::new(files.len() as u64);
    //process all the beatmaps in parallel if they are from Standard and save them in output vector.
    let new_data: Vec<FullData> = files.par_iter()
        .fold(|| Vec::with_capacity(files.len()), |mut acc, path| {
            bar.inc(1);
            let md5 = calculate_md5(path);
            if !already_processed.contains(&md5) {
                if let Ok(map) = Beatmap::from_path(path) {
                    if map.mode == GameMode::Osu {
                        if let Ok(song) = process_stats(&map.timing_points,&map.hit_objects,map.cs,options.min_stream_distance,options.min_jump_distance){
                            //using my function to parse title version map id and map set id because rosu pp doesnt have -_-
                            let (title1,creator1, version1, beatmap_id1, beatmap_set_id1) = read_osu_file(path);
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
                                creator: creator1,
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
    new_data
}
#[inline(always)]
fn are_options_equal(path: String)->bool{
    let options_config_md5 = calculate_md5(&path);
    let data_options_config_md5 = calculate_md5(format!("data\\{}",path).as_str());
    options_config_md5 == data_options_config_md5
}
fn main() {
    let options_config_path = String::from("options_config.txt");
    let columns_config_vec = init_columns_config(String::from("columns_config.txt"));
    let options_config = init_options_config(options_config_path.clone());
    let mut old_data = Vec::new(); 
    if let Ok(data) = read_main_data_csv(){
        old_data=data;
    }
    let are_options_equal_bool = are_options_equal(options_config_path.clone());
    let already_processed:HashSet<String> = if are_options_equal_bool {
        old_data.iter().map(|item| item.md5.clone()).collect()
    } else {
        HashSet::new()   
    };
    let mut songs_path = String::new();
    if options_config.file_path.len()< 1{
        print!("Enter Osu Song path (example D:\\osu\\Songs): ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut songs_path)
            .expect("failed to read from stdin");
    }
    else {songs_path=options_config.file_path.clone()}

    process_beatmaps(songs_path,&already_processed,&old_data,&columns_config_vec,options_config,are_options_equal_bool,options_config_path);
    end();
}
fn end(){
    // !!! We make the user press enter to leave the program !!!
    let mut input = String::new();
    print!("Press enter to exit...");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).expect("Failed to read line");
}