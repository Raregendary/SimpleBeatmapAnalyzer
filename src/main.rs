

use indicatif::ProgressBar;
mod osufile_reader;
use osufile_reader::read_osu_file;
mod beatmap_stat_processor;
use beatmap_stat_processor::{process_stats, SongParams};
mod savedata_manager;
use savedata_manager::{calculate_md5, load_bin_data};
use jwalk::WalkDir as wd;
use rayon::prelude::*;
use rosu_pp::{Beatmap, BeatmapExt,GameMode, PerformanceAttributes};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::hash::Hash;
use std::io::Write;
use std::sync::{Arc, RwLock};
use std::{collections::HashSet, io};
use std::path::{PathBuf, self};
use std::time::Instant;
mod full_data_struct;
use full_data_struct::{FullData, FullDataEnum, SavableFullData, normal_to_serialized_beatmap, serialized_to_normal_beatmap};
use crate::savedata_manager::{data_save_manager};
mod options_config_manager;
use options_config_manager::{init_options_config, OptionsConfig, compare_options_config, OptionsConfigComparedBools, ExactlyTheSame};
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
fn process_new_beatmap(path:&str,md5:String,options: &OptionsConfig) -> Result<SavableFullData, Box<dyn Error>>{
    if let Ok(map) = Beatmap::from_path(path) {
        if map.mode != GameMode::Osu {
            return Ok(SavableFullData{
                beatmap: normal_to_serialized_beatmap(map),
                title: String::new(),
                version: String::new(),
                beatmap_id: String::new(),
                beatmap_set_id: String::new(),
                creator: String::new(),
                nm_stars: 0.0,
                dt_stars: 0.0,
                hr_stars: 0.0,
                nm_pp: 0.0,
                dt_pp: 0.0,
                hr_pp: 0.0,
                song: SongParams::new_initialized(),
                md5: md5 
            });
        }
        if let Ok(song) = process_stats(&map.timing_points,&map.hit_objects,map.cs,options.min_stream_distance,options.min_jump_distance){
            //using my function to parse title version map id and map set id because rosu pp doesnt have -_-
            let (title1,creator1, version1, beatmap_id1, beatmap_set_id1) = read_osu_file(path);
            let nm = map.pp().accuracy(99.0).calculate();
            let dt = map.pp().mods(64).accuracy(99.0).calculate();
            let hr = map.pp().mods(16).accuracy(99.0).calculate();
            return Ok(SavableFullData{
                beatmap: normal_to_serialized_beatmap(map),
                title: title1,
                version: version1,
                beatmap_id: beatmap_id1,
                beatmap_set_id: beatmap_set_id1,
                creator: creator1,
                nm_stars: nm.stars() as f32,
                dt_stars: dt.stars() as f32,
                hr_stars: hr.stars() as f32,
                nm_pp: nm.pp() as f32,
                dt_pp: dt.pp() as f32,
                hr_pp: hr.pp() as f32,
                song: song,
                md5: md5 
            });
        }
    }
    Err("Failed to process new beatmap".into())
}
#[inline(always)]
fn update_mut_savable_fulldata(item: &mut SavableFullData,options: &OptionsConfig,compared_options: &OptionsConfigComparedBools,are_options_equal_bool: bool) {
    if !are_options_equal_bool{
        let normal_beatmap = serialized_to_normal_beatmap(item.beatmap.clone());
        if !compared_options.are_jump_distance_the_same || !compared_options.are_stream_distance_the_same{
            if let Ok(song) = process_stats(
                &normal_beatmap.timing_points,&normal_beatmap.hit_objects,normal_beatmap.cs,
                options.min_stream_distance,options.min_jump_distance){
                item.song = song;
            }
        }
    }
}

#[inline(always)]
fn data_processor_new(files: &[String], old_data: &mut HashMap<String,SavableFullData>, options: &OptionsConfig, compared_options: &OptionsConfigComparedBools, are_options_equal_bool: bool) {
    let old_data = Arc::new(RwLock::new(old_data));
    let bar = ProgressBar::new(files.len() as u64);
    files.par_iter().for_each(|file| {
        bar.inc(1);
        let md5 = calculate_md5(file);
        let mut old_data = old_data.write().unwrap();
        if let Some(old_savable_fulldata) = old_data.get_mut(&md5) {
            update_mut_savable_fulldata(old_savable_fulldata,options,compared_options,are_options_equal_bool);
        }
        else{
            if let Ok(savable_full_data) =process_new_beatmap(file,md5.clone(),options){
                old_data.insert(md5, savable_full_data);
            }
        }
    });
    bar.finish();
}
#[inline(always)]
fn process_beatmaps_new(songs_path:String,old_data: &mut HashMap<String,SavableFullData>, columns_config_vec: &[FullDataEnum],
    options: OptionsConfig,compared_options: OptionsConfigComparedBools,options_config_path:String,are_options_equal_bool:bool){
    //Try to get all the .osu files
    if let Ok(files) = find_osu_files6(songs_path.trim().into()){
        println!("Found {} osu beatmaps.\nProcessing Osu! Standard beatmaps:",files.len());
        let start = Instant::now();
        data_processor_new(&files,old_data,&options,&compared_options,are_options_equal_bool);

        let duration = start.elapsed();
        println!("Processed {} Osu! Standard beatmaps in: {:?}\t({} beatmaps per second)",old_data.len(), duration,old_data.len() / duration.as_secs().max(1) as usize);
        //okay do tuka sme
        
        
        if let Ok(path) = data_save_manager(old_data,columns_config_vec,options_config_path){
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
    let options_config_path = String::from("options_config.txt");
    let columns_config_vec = init_columns_config(String::from("columns_config.txt"));
    let options_config = init_options_config(options_config_path.clone());
    let compared_options = compare_options_config(
        &options_config,
        &init_options_config(format!("data\\{}",options_config_path))
    );
    let mut old_data = HashMap::<String,SavableFullData>::new(); 
    if let Ok(data) = load_bin_data(){
        old_data=data;
    }
    let are_options_equal_bool = compared_options.are_the_same();
    let mut songs_path = String::new();
    if options_config.file_path.len()< 1{
        print!("Enter Osu Song path (example D:\\osu\\Songs): ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut songs_path)
            .expect("failed to read from stdin");
    }
    else {songs_path=options_config.file_path.clone()}

    process_beatmaps_new(songs_path,&mut old_data,&columns_config_vec,options_config,compared_options,options_config_path,are_options_equal_bool);
    end();
}
fn end(){
    // !!! We make the user press enter to leave the program !!!
    let mut input = String::new();
    print!("Press enter to exit...");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).expect("Failed to read line");
}