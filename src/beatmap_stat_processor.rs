use std::{collections::HashMap, error::Error};
use rosu_pp::{beatmap::TimingPoint, parse::HitObject};
use serde::{Serialize, Deserialize};

#[inline(always)]
pub fn process_stats(sorted_bpms: &[TimingPoint], xyt: &[HitObject], cs: f32,stream_spacing: f32,jump_spacing: f32) -> Result<SongParams, Box<dyn Error>> {
    //first circle is not considered anything, also for sliders only the initial circle is calculated/counted
    if xyt.len() <= 3 || sorted_bpms.len() == 0 {/*skip maps with less than 3 elements xD and no timing points */return Err("map too short".into())}
    let mut beatmap_playable_length = 0.0;
    let mut map = HashMap::new();
    //calculate the diameter of a circle.Used to calculate how spaced are they later
    let r2 = (((54.4 - 4.48 * cs) * 1000.0).round() / 1000.0 ) * 2.0;
    let mut time_last: f64 = 0.0;
    let mut x_last: f32 = 0.0;
    let mut y_last: f32 = 0.0;
    //counters for the streams / jumps etc.
    let (mut counter_one_fourth,mut counter_one_twoth) = (0,0);
    let (mut n_burst,mut n_stream,mut n_deathstream) = (0,0,0);
    let (mut n_short,mut n_mid,mut n_long) = (0,0,0);
    let (mut n_doubles,mut n_triples,mut n_quads) = (0,0,0);
    let mut bpm_index = 0;
    let bpm_max_index= sorted_bpms.len()-1;
    let mut current_bpm_ms_time;
    //we take the values of the first object
    if let Some(first_element) = xyt.first() {
        x_last = first_element.pos.x;
        y_last = first_element.pos.y;
        time_last = first_element.start_time;
        beatmap_playable_length = time_last;
    }
    //variables after v0.9.0
    let mut longest_stream = 0;
    let mut stream100_counter =0;
    let mut avarage_jump_speed_counter: f32 = 0.0;
    let mut avarage_jump_distance_counter: f32 = 0.0;
    let mut jumps_counter: u32 = 0;
    for i in xyt.iter().skip(1){
        let x = i.pos.x;
        let y = i.pos.y;
        let start_time = i.start_time;
        // calculate the distance between the 2 circles and remove the diameter so we have  the edge to edge spacing
        let d_r =  (((x_last - x) * (x_last - x)) + ((y_last - y) * (y_last - y))).sqrt() - r2;
        // get current bpm
        if bpm_index == bpm_max_index{
            current_bpm_ms_time = unsafe { *sorted_bpms.get_unchecked(bpm_index) }.beat_len;
        }
        else {
            loop{
                if bpm_index == bpm_max_index{
                    current_bpm_ms_time = unsafe { *sorted_bpms.get_unchecked(bpm_index) }.beat_len;
                    break;
                }                                            
                else if unsafe { *sorted_bpms.get_unchecked(bpm_index+1) }.time > start_time {
                    current_bpm_ms_time = unsafe { *sorted_bpms.get_unchecked(bpm_index) }.beat_len;
                    break;
                }
                bpm_index+=1;
            }
        }
        //vvvvvvvvvvvvvvv-MOST COMMON BPM 
        let counter = map.entry((current_bpm_ms_time*1000.0) as i32).or_insert(0);
        *counter += 1;
        //^^^^^^^^^^^^^^^-MOST COMMON BPM 
        //this is a ration that if the map is 1/4th and we are in 1/4th gives us around 1 and if we are at 1/2 it gives us around 2
        let time_deviser_ratio = (start_time-time_last) / (0.25*current_bpm_ms_time);
        //here we check for streams in 1/4 from a 1/4 map and spacing less than 16 pixels edge to edge or overlaping
        if d_r - 16.0 <= 0.0 && time_deviser_ratio > 0.9 && time_deviser_ratio < 1.1 {
            counter_one_fourth+=1;
        }
        else if counter_one_fourth > 0 {
            counter_one_fourth+=1;//adding the first element
            if longest_stream<counter_one_fourth{
                longest_stream = counter_one_fourth;
            }
            if counter_one_fourth == 2 {
                n_doubles   += 2; 
            } else if counter_one_fourth == 3{
                n_triples   += 3;
                n_burst     += 3;
            } else if counter_one_fourth == 4{
                n_quads     += 4;
                n_burst     += 4;
            } else if counter_one_fourth <= 11{
                n_burst     += counter_one_fourth;
            }else if counter_one_fourth <= 32{
                n_stream    += counter_one_fourth;
            } else {
                n_deathstream += counter_one_fourth;
                if counter_one_fourth >= 100{
                    stream100_counter+=1;
                }
            }
            counter_one_fourth = 0;
        } 
        if d_r - 110.0 > 0.0 && time_deviser_ratio > 1.9 && time_deviser_ratio < 2.1{
            // here we check for jumps in 1/2 from a 1/4th map and spacing more then 110 pixes edge to edge      
            counter_one_twoth+=1;
            avarage_jump_distance_counter += d_r;
            avarage_jump_speed_counter += (1000.0*d_r) / (start_time - time_last) as f32;
        }  
        else if counter_one_twoth >= 2{
            jumps_counter+=counter_one_twoth+1;
            if counter_one_twoth <= 10{
                n_short += counter_one_twoth + 1;
            }
            else if counter_one_twoth >= 11 && counter_one_twoth <= 31{
                n_mid += counter_one_twoth + 1;
            }
            else{
                n_long += counter_one_twoth + 1;
            } 
            counter_one_twoth = 0;
        }
        //  Asigning last elements for next iteration
        time_last=start_time;
        x_last = x;
        y_last = y;
    }
    //---------------------Calculating if the patern persist till the end of the map -----------------
    if counter_one_fourth > 0 {
        counter_one_fourth+=1;//adding the first element
        if longest_stream<counter_one_fourth{
            longest_stream = counter_one_fourth;
        }
        if counter_one_fourth == 2 {
            n_doubles   += 2; 
        } else if counter_one_fourth == 3{
            n_triples   += 3;
            n_burst     += 3;
        } else if counter_one_fourth == 4{
            n_quads     += 4;
            n_burst     += 4;
        } else if counter_one_fourth <= 11{
            n_burst     += counter_one_fourth;
        }else if counter_one_fourth <= 32{
            n_stream    += counter_one_fourth;
        } else {
            n_deathstream += counter_one_fourth;
            if counter_one_fourth >= 100{
                stream100_counter+=1;
            }
        }
    } 
    if counter_one_twoth >= 2{
        jumps_counter+=counter_one_twoth+1;
        if counter_one_twoth <= 10{
            n_short += counter_one_twoth + 1;
        }
        else if counter_one_twoth >= 11 && counter_one_twoth <= 31{
            n_mid += counter_one_twoth + 1;
        }
        else{
            n_long += counter_one_twoth + 1;
        }
    }
    //^^^^^^^^^^^^^^^^^^^^^^^^^^^Calculating if the patern persist till the end of the map^^^^^^^^^^^^^^^^^^^^^^^^^
    let length = xyt.len() as f32;
    avarage_jump_speed_counter = avarage_jump_speed_counter/jumps_counter as f32; // calculating the avg for the entire map
    avarage_jump_distance_counter = avarage_jump_distance_counter/jumps_counter as f32; // calculating the avg for the entire map

    let most_common_bpm = map.into_iter().max_by_key(|&(_, count)| count).map(|(val, _)| val).unwrap();
    beatmap_playable_length = ((time_last - beatmap_playable_length)/1000.0).round();
    let jump_value = n_short as f32 + n_mid as f32 * 1.5 + n_long as f32 * 2.0;
    let steam_value = n_burst as f32 + n_stream as f32 * 1.5 + n_deathstream as f32 * 2.0;
    Ok(SongParams{
        playable_length : beatmap_playable_length as i32,
        bpm: (60000.0 / (most_common_bpm as f32 / 1000.0)).round() as i32,
        doubles: n_doubles as f32/length * 100.0,
        triples:n_triples as f32/length * 100.0,
        bursts: n_burst as f32/length * 100.0,
        streams: n_stream as f32/length * 100.0,
        deathstreams: n_deathstream as f32/length * 100.0,
        short_jumps: n_short as f32/length * 100.0,
        mid_jumps: n_mid as f32/length * 100.0,
        long_jumps: n_long as f32/length * 100.0,
        quads: n_quads as f32/length * 100.0,
        fcdbi: ((n_burst-n_triples-n_quads) as f32 +(n_doubles as f32*1.75) + (n_triples as f32 * 1.5) + (n_quads as f32 * 1.75) - jump_value - (n_stream as f32 * 1.35) - (n_deathstream as f32 * 1.7)) / length,
        si: (steam_value - jump_value - (n_doubles as f32 * 0.5)) / length,
        ji: (jump_value - steam_value - (n_doubles as f32 * 0.5)) / length,
        longest_stream: longest_stream,
        streams100: stream100_counter,
        avg_jump_distance: (if avarage_jump_distance_counter.is_finite() {avarage_jump_distance_counter as u32}else{0}),
        avg_jump_speed: (if avarage_jump_speed_counter.is_finite() {avarage_jump_speed_counter as u32} else {0}),
        }
    )
} 
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct SongParams {
    pub playable_length: i32,
    pub bpm: i32,           // most common bpm
    pub doubles: f32,
    pub triples: f32,
    pub bursts: f32,        // 3-12     1/4th
    pub streams: f32,       // 13-32    1/4th
    pub deathstreams: f32,  // 33+      1/4th
    pub short_jumps: f32,   // 3-12     1/2th
    pub mid_jumps: f32,     // 13-32    1/2th
    pub long_jumps: f32,    // 33+      1/2th
    pub quads: f32,
    pub fcdbi: f32,//FINGER CONTROL DOUBLE BURSTS INDEX
    pub si: f32,//Stream index
    pub ji: f32,//Jump index
    pub longest_stream: u32,
    pub streams100: u32,
    pub avg_jump_distance: u32, // Average distance between jumps in pixels
    pub avg_jump_speed: u32, // Average speed of jumps in ms
}
impl SongParams {
    #[inline(always)]
    pub fn new_initialized() -> Self {
        Self {
            playable_length: 0,
            bpm: 0,
            doubles: 0.0,
            triples: 0.0,
            bursts: 0.0,
            streams: 0.0,
            deathstreams: 0.0,
            short_jumps: 0.0,
            mid_jumps: 0.0,
            long_jumps: 0.0,
            quads: 0.0,
            fcdbi: 0.0,
            si: 0.0,
            ji: 0.0,
            longest_stream: 0,
            streams100: 0,
            avg_jump_distance: 0,
            avg_jump_speed: 0,
        }
    }
}

#[allow(dead_code)]
#[inline(always)]
fn most_common(v: &[i32]) -> i32 {
    let mut counts = HashMap::new();
    for &value in v {
        *counts.entry(value).or_insert(0) += 1;
    }
    counts.into_iter().max_by_key(|&(_, count)| count).map(|(val, _)| val).unwrap()
}