use std::{str::FromStr, default};
extern crate rosu_pp;
use rosu_pp::{Beatmap, PerformanceAttributes, GameMode, SortedVec, parse::{HitObject, Pos2}};
use serde::{Deserialize,Serialize};

use strum_macros::EnumIter;

use crate::beatmap_stat_processor::SongParams;



#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct FullData {
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "DifName")]
    pub version: String,
    #[serde(rename = "MapID")]
    pub beatmap_id: String,
    #[serde(rename = "MapSetID")]
    pub beatmap_set_id: String,
    #[serde(rename = "CS")]
    pub cs: f32,
    #[serde(rename = "AR")]
    pub ar: f32,
    #[serde(rename = "OD")]
    pub od: f32,
    #[serde(rename = "HP")]
    pub hp: f32,
    #[serde(rename = "Stars")]
    pub stars_nm: f32,
    #[serde(rename = "DT_Stars")]
    pub stars_dt: f32,
    #[serde(rename = "HR_Stars")]
    pub stars_hr: f32,
    #[serde(rename = "NM_99")]
    pub nm: f32,
    #[serde(rename = "DT_99")]
    pub dt: f32,
    #[serde(rename = "HR_99")]
    pub hr: f32,
    #[serde(rename = "PlayableLength")]
    pub playable_length: i32,
    #[serde(rename = "BPM")]
    pub bpm: i32,           // most common bpm
    #[serde(rename = "Doubles")]
    pub doubles: f32,
    #[serde(rename = "Triples")]
    pub triples: f32,
    #[serde(rename = "Bursts")]
    pub bursts: f32,        // 3-12     1/4th
    #[serde(rename = "Streams")]
    pub streams: f32,       // 13-32    1/4th
    #[serde(rename = "DeathStreams")]
    pub deathstreams: f32,  // 33+      1/4th
    #[serde(rename = "ShortJumps")]
    pub short_jumps: f32,   // 3-12     1/2th
    #[serde(rename = "MidJumps")]
    pub mid_jumps: f32,     // 13-32    1/2th
    #[serde(rename = "Longjumps")]
    pub long_jumps: f32,    // 33+      1/2th
    #[serde(rename = "Quads")]
    pub quads: f32,
    #[serde(rename = "FCDBI")]
    pub fcdbi: f32,//FINGER CONTROL DOUBLE BURSTS INDEX
    #[serde(rename = "SI")]
    pub si: f32,//Stream index
    #[serde(rename = "JI")]
    pub ji: f32,//Jump index
    #[serde(rename = "LongestStream")]
    pub longest_stream: u32,
    #[serde(rename = "Streams100")]
    pub streams100: u32, // how much 100 or higher note streams are in the map as counter
    #[serde(rename = "AvgJumpsDistance")]
    pub avg_jump_distance: u32, // Avarage jump distance in pixels from all the jumps on the map (even if 2 objects only)
    #[serde(rename = "AvgJumpsSpeed")]
    pub avg_jump_speed: u32,  // Avarage jump speed in pixels/sec from all the jumps on the map (even if 2 objects only)
    //0.9.2
    #[serde(rename = "Creator")]
    pub creator: String,
    #[serde(rename = "MD5")]
    pub md5: String,
}
pub trait FullDataTrait {
    fn get_string(&self, field: &FullDataEnum) -> String;
}

impl FullDataTrait for FullData {
    #[inline(always)]
    fn get_string(&self, field: &FullDataEnum) -> String {
        match field {
            FullDataEnum::Title => self.title.to_string(),
            FullDataEnum::DifName => self.version.to_string(),
            FullDataEnum::MapID => self.beatmap_id.to_string(),
            FullDataEnum::Stars => format!("{:.2}", self.stars_nm),
            FullDataEnum::BPM => self.bpm.to_string(),
            FullDataEnum::Bursts => format!("{:.2}", self.bursts),
            FullDataEnum::Streams => format!("{:.2}", self.streams),
            FullDataEnum::DeathStreams => format!("{:.2}", self.deathstreams),
            FullDataEnum::ShortJumps => format!("{:.2}", self.short_jumps),
            FullDataEnum::MidJumps => format!("{:.2}", self.mid_jumps),
            FullDataEnum::Longjumps => format!("{:.2}", self.long_jumps),
            FullDataEnum::Doubles => format!("{:.2}", self.doubles),
            FullDataEnum::Triples => format!("{:.2}", self.triples),
            FullDataEnum::SI => format!("{:.2}", self.si),
            FullDataEnum::JI => format!("{:.2}", self.ji),
            FullDataEnum::FCDBI => format!("{:.2}", self.fcdbi),
            FullDataEnum::PlayableLength => self.playable_length.to_string(),
            FullDataEnum::CS => self.cs.to_string(),
            FullDataEnum::AR => self.ar.to_string(),
            FullDataEnum::OD => self.od.to_string(),
            FullDataEnum::HP => self.hp.to_string(),
            FullDataEnum::NM_99 => format!("{:.2}", self.nm),
            FullDataEnum::DT_99 => format!("{:.2}", self.dt),
            FullDataEnum::HR_99 => format!("{:.2}", self.hr),
            FullDataEnum::DT_Stars => format!("{:.2}", self.stars_dt),
            FullDataEnum::HR_Stars => format!("{:.2}", self.stars_hr),
            FullDataEnum::DT_BPM =>((self.bpm as f32 * 1.5).ceil() as i32).to_string(),
            FullDataEnum::DT_AR => apply_dt_to_ar(self.ar).to_string(),
            FullDataEnum::HR_AR => format!("{:.2}", (self.ar * 1.4).min(10.0)),
            FullDataEnum::HR_CS => format!("{:.2}", (self.cs * 1.3).min(10.0)),
            FullDataEnum::Quads => format!("{:.2}", self.quads),
            FullDataEnum::MapSetID => self.beatmap_set_id.to_string(),
            //0.9.1
            FullDataEnum::LongestStream => self.longest_stream.to_string(),
            FullDataEnum::Streams100 => self.streams100.to_string(),
            FullDataEnum::AvgJumpsDistance => self.avg_jump_distance.to_string(),
            FullDataEnum::AvgJumpsSpeed => format!("{:.2}", self.avg_jump_speed),
            //0.9.2
            FullDataEnum::Creator => self.creator.to_string(),
            FullDataEnum::OsuWebLink => if self.beatmap_id.len()>1 {format!("=HYPERLINK(\"https://osu.ppy.sh/b/{}\")", self.beatmap_id)} else { "".into()},
            FullDataEnum::OsuDirect => if self.beatmap_id.len()>1 {format!("=HYPERLINK(\"osu://b/{}\")", self.beatmap_id)} else { "".into()},
            FullDataEnum::MD5 => self.md5.to_string(),
            
            _ => "".into(),
        }
    }
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






#[derive(Debug,EnumIter)]
pub enum FullDataEnum {
    Title = 0 ,
    DifName,
    MapID,
    Stars,
    BPM,
    Bursts,
    Streams,
    DeathStreams,
    ShortJumps,
    MidJumps,
    Longjumps,
    Doubles,
    Triples,
    SI,
    JI, 
    FCDBI,
    PlayableLength,
    CS,
    AR,
    OD,
    HP,
    NM_99,
    DT_99,
    HR_99,
    DT_Stars,
    HR_Stars,
    DT_BPM,
    DT_AR,
    HR_AR,
    HR_CS,
    Quads,
    MapSetID,
    LongestStream,
    Streams100,
    AvgJumpsDistance,
    AvgJumpsSpeed,
    Creator,
    OsuDirect,
    OsuWebLink,
    MD5
}
impl FromStr for FullDataEnum {
    type Err = ();
    #[inline(always)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Title" => Ok(FullDataEnum::Title),
            "DifName" => Ok(FullDataEnum::DifName),
            "MapID" => Ok(FullDataEnum::MapID),
            "Stars" => Ok(FullDataEnum::Stars),
            "BPM" => Ok(FullDataEnum::BPM),
            "Bursts" => Ok(FullDataEnum::Bursts),
            "Streams" => Ok(FullDataEnum::Streams),
            "DeathStreams" => Ok(FullDataEnum::DeathStreams),
            "ShortJumps" => Ok(FullDataEnum::ShortJumps),
            "MidJumps" => Ok(FullDataEnum::MidJumps),
            "Longjumps" => Ok(FullDataEnum::Longjumps),
            "Doubles" => Ok(FullDataEnum::Doubles),
            "Triples" => Ok(FullDataEnum::Triples),
            "SI" => Ok(FullDataEnum::SI),
            "JI" => Ok(FullDataEnum::JI),
            "FCDBI" => Ok(FullDataEnum::FCDBI),
            "PlayableLength" => Ok(FullDataEnum::PlayableLength),
            "CS" => Ok(FullDataEnum::CS),
            "AR" => Ok(FullDataEnum::AR),
            "OD" => Ok(FullDataEnum::OD),
            "HP" => Ok(FullDataEnum::HP),
            "NM_99"=>Ok(FullDataEnum::NM_99), 
            "DT_99"=>Ok(FullDataEnum::DT_99), 
            "HR_99"=>Ok(FullDataEnum::HR_99), 
            "DT_Stars"=>Ok(FullDataEnum::DT_Stars), 
            "HR_Stars"=>Ok(FullDataEnum::HR_Stars), 
            "DT_BPM"=>Ok(FullDataEnum::DT_BPM), 
            "DT_AR"=>Ok(FullDataEnum::DT_AR), 
            "HR_AR"=>Ok(FullDataEnum::HR_AR), 
            "HR_CS"=>Ok(FullDataEnum::HR_CS), 
            "Quads"=>Ok(FullDataEnum::Quads), 
            "MapSetID"=>Ok(FullDataEnum::MapSetID), 
            "LongestStream"=>Ok(FullDataEnum::LongestStream), 
            "Streams100"=>Ok(FullDataEnum::Streams100), 
            "AvgJumpsDistance"=>Ok(FullDataEnum::AvgJumpsDistance), 
            "AvgJumpsSpeed"=>Ok(FullDataEnum::AvgJumpsSpeed), 
            "Creator"=>Ok(FullDataEnum::Creator),
            "OsuDirect"=>Ok(FullDataEnum::OsuDirect),
            "OsuWebLink"=>Ok(FullDataEnum::OsuWebLink),
            "MD5"=>Err(()),//Always added in the end :)        ---Ok(FullDataEnum::MD5),

             _=>Err(()),

        }
    }
}

#[derive(Debug, Clone,Serialize,Deserialize)]
pub struct SavableFullData {
    pub beatmap: Beatmap_Serialized,
    pub title: String,
    pub version: String,
    pub beatmap_id: String,
    pub beatmap_set_id: String,
    pub creator: String,
    pub nm_stars: f32,
    pub dt_stars: f32,
    pub hr_stars: f32,
    pub nm_pp: f32,
    pub dt_pp: f32,
    pub hr_pp: f32,
    pub song: SongParams,
    pub md5: String,
}
pub trait SavableFullDataTrait {
    fn get_string(&self, field: &FullDataEnum) -> String;
}

impl SavableFullDataTrait for SavableFullData {
    #[inline(always)]
    fn get_string(&self, field: &FullDataEnum) -> String {
        match field {
            FullDataEnum::Title => self.title.to_string(),
            FullDataEnum::DifName => self.version.to_string(),
            FullDataEnum::MapID => self.beatmap_id.to_string(),
            FullDataEnum::Stars => format!("{:.2}", self.nm_stars),
            FullDataEnum::BPM => self.song.bpm.to_string(),
            FullDataEnum::Bursts => format!("{:.2}", self.song.bursts),
            FullDataEnum::Streams => format!("{:.2}", self.song.streams),
            FullDataEnum::DeathStreams => format!("{:.2}", self.song.deathstreams),
            FullDataEnum::ShortJumps => format!("{:.2}", self.song.short_jumps),
            FullDataEnum::MidJumps => format!("{:.2}", self.song.mid_jumps),
            FullDataEnum::Longjumps => format!("{:.2}", self.song.long_jumps),
            FullDataEnum::Doubles => format!("{:.2}", self.song.doubles),
            FullDataEnum::Triples => format!("{:.2}", self.song.triples),
            FullDataEnum::SI => format!("{:.2}", self.song.si),
            FullDataEnum::JI => format!("{:.2}", self.song.ji),
            FullDataEnum::FCDBI => format!("{:.2}", self.song.fcdbi),
            FullDataEnum::PlayableLength => self.song.playable_length.to_string(),
            FullDataEnum::CS => self.beatmap.cs.to_string(),
            FullDataEnum::AR => self.beatmap.ar.to_string(),
            FullDataEnum::OD => self.beatmap.od.to_string(),
            FullDataEnum::HP => self.beatmap.hp.to_string(),
            FullDataEnum::NM_99 => format!("{:.2}", self.nm_pp),
            FullDataEnum::DT_99 => format!("{:.2}", self.dt_pp),
            FullDataEnum::HR_99 => format!("{:.2}", self.hr_pp),
            FullDataEnum::DT_Stars => format!("{:.2}", self.dt_stars),
            FullDataEnum::HR_Stars => format!("{:.2}", self.hr_stars),
            FullDataEnum::DT_BPM =>((self.song.bpm as f32 * 1.5).ceil() as i32).to_string(),
            FullDataEnum::DT_AR => apply_dt_to_ar(self.beatmap.ar).to_string(),
            FullDataEnum::HR_AR => format!("{:.2}", (self.beatmap.ar * 1.4).min(10.0)),
            FullDataEnum::HR_CS => format!("{:.2}", (self.beatmap.cs * 1.3).min(10.0)),
            FullDataEnum::Quads => format!("{:.2}", self.song.quads),
            FullDataEnum::MapSetID => self.beatmap_set_id.to_string(),
            //0.9.1
            FullDataEnum::LongestStream => self.song.longest_stream.to_string(),
            FullDataEnum::Streams100 => self.song.streams100.to_string(),
            FullDataEnum::AvgJumpsDistance => self.song.avg_jump_distance.to_string(),
            FullDataEnum::AvgJumpsSpeed => format!("{:.2}", self.song.avg_jump_speed),
            //0.9.2
            FullDataEnum::Creator => self.creator.to_string(),
            FullDataEnum::OsuWebLink => if self.beatmap_id.len()>1 {format!("=HYPERLINK(\"https://osu.ppy.sh/b/{}\")", self.beatmap_id)} else { String::new()},
            FullDataEnum::OsuDirect => if self.beatmap_id.len()>1 {format!("=HYPERLINK(\"osu://b/{}\")", self.beatmap_id)} else { String::new()},
            FullDataEnum::MD5 => self.md5.to_string(),
            _ => String::new(),
        }
    }
}

#[derive(Clone, Default, Debug,Serialize,Deserialize)]
pub struct Beatmap_Serialized {
    /// The game mode.
    pub mode: GameMode_Serialized,
    /// The version of the .osu file.
    pub version: u8,

    /// The amount of circles.
    pub n_circles: u32,
    /// The amount of sliders.
    pub n_sliders: u32,
    /// The amount of spinners.
    pub n_spinners: u32,

    /// The approach rate.
    pub ar: f32,
    /// The overall difficulty.
    pub od: f32,
    /// The circle size.
    pub cs: f32,
    /// The health drain rate.
    pub hp: f32,
    /// Base slider velocity in pixels per beat
    pub slider_mult: f64,
    /// Amount of slider ticks per beat.
    pub tick_rate: f64,
    /// All hitobjects of the beatmap.
    pub hit_objects: Vec<HitObject_Serialized>,
    /// Store the sounds for all objects in their own Vec to minimize the struct size.
    /// Hitsounds are only used in osu!taiko in which they represent color.
    pub sounds: Vec<u8>,

    /// Timing points that indicate a new timing section.
    pub timing_points: Vec<TimingPoint_Serialized>,

    /// Timing point for the current timing section.
    pub difficulty_points: Vec<DifficultyPoint_Serialized>,

    /// Control points for effect sections.
    pub effect_points: Vec<EffectPoint_Serialized>,

    /// The stack leniency that is used to calculate
    /// the stack offset for stacked positions.
    pub stack_leniency: f32,

    /// All break points of the beatmap.
    pub breaks: Vec<Break_Serialized>,
}


#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq,Serialize,Deserialize,Default)]
pub enum GameMode_Serialized {
    /// osu!standard
    #[default]
    Osu = 0,
    /// osu!taiko
    Taiko = 1,
    /// osu!catch
    Catch = 2,
    /// osu!mania
    Mania = 3,
}

#[derive(Clone, Debug, PartialEq,Serialize,Deserialize)]
pub struct HitObject_Serialized {
    /// The position of the object.
    pub pos: Pos2_Serialized,
    /// The start time of the object.
    pub start_time: f64,
    /// The type of the object.
    pub kind: HitObjectKind_Serialized,
}

#[derive(Debug,Clone, Copy, Default, PartialEq,Serialize,Deserialize)]
pub struct Pos2_Serialized {
    /// Position on the x-axis.
    pub x: f32,
    /// Position on the y-axis.
    pub y: f32,
}

/// Further data related to specific object types.
#[derive(Clone, Debug, PartialEq,Serialize,Deserialize)]
pub enum HitObjectKind_Serialized  {
    /// A circle object.
    Circle,
    /// A full slider object.
    Slider {
        /// Total length of the slider in pixels.
        pixel_len: Option<f64>,
        /// The amount of repeat points of the slider.
        repeats: usize,
        /// The control points of the slider.
        control_points: Vec<PathControlPoint_Serialized>,
        /// Sample sounds for the slider head, end, and repeat points.
        /// Required for converts.
        edge_sounds: Vec<u8>,
    },
    /// A spinner object.
    Spinner {
        /// The end time of the spinner.
        end_time: f64,
    },
    /// A hold note object for osu!mania.
    Hold {
        /// The end time of the hold object.
        end_time: f64,
    },
}

  /// Control point for slider curve calculation
  #[derive(Copy, Clone, Debug, Default, PartialEq,Serialize,Deserialize)]
  pub struct PathControlPoint_Serialized {
      /// Control point position.
      pub pos: Pos2_Serialized,
      /// Path type of the control point.
      /// Only present for the first element of each segment.
      pub kind: Option<PathType_Serialized>,
  }

   /// The type of curve of a slider.
   #[allow(missing_docs)]
   #[derive(Copy, Clone, Debug, Eq, PartialEq,Serialize,Deserialize)]
   pub enum PathType_Serialized {
       Catmull = 0,
       Bezier = 1,
       Linear = 2,
       PerfectCurve = 3,
   }

   /// New rhythm speed change.
#[derive(Copy, Clone, Debug, PartialEq,Serialize,Deserialize)]
pub struct TimingPoint_Serialized {
    /// The beat length for this timing section
    pub beat_len: f64,
    /// The start time of this timing section
    pub time: f64,
}

/// [`TimingPoint`] that depends on a previous one.
#[derive(Copy, Clone, Debug, PartialEq,Serialize,Deserialize)]
pub struct DifficultyPoint_Serialized  {
    /// The time at which the control point takes effect.
    pub time: f64,
    /// The slider velocity at this control point.
    pub slider_vel: f64,
    /// Legacy BPM multiplier that introduces floating-point errors for rulesets that depend on it.
    pub bpm_mult: f64,
    /// Whether or not slider ticks should be generated at this control point.
    /// This exists for backwards compatibility with maps that abuse NaN
    /// slider velocity behavior on osu!stable (e.g. /b/2628991).
    pub generate_ticks: bool,
}
/// Control point storing effects and their timestamps.
#[derive(Copy, Clone, Debug, PartialEq,Serialize,Deserialize)]
pub struct EffectPoint_Serialized {
    /// The time at which the control point takes effect.
    pub time: f64,
    /// Whether this control point enables Kiai mode.
    pub kiai: bool,
}

/// A break point of a [`Beatmap`](crate::beatmap::Beatmap).
#[derive(Copy, Clone, Debug, Default, PartialEq,Serialize,Deserialize)]
pub struct Break_Serialized {
    /// Start timestamp of the break.
    pub start_time: f64,
    /// End timestamp of the break.
    pub end_time: f64,
}

// ----------------------------- TO NORMAL -------------------------------------------------------------------------------------------------------------------------------------
#[inline(always)]
pub fn match_gamemodes_to_normal(gamemode: GameMode_Serialized)->GameMode{
    match gamemode {
        GameMode_Serialized::Osu => GameMode::Osu,
        GameMode_Serialized::Taiko => GameMode::Taiko,
        GameMode_Serialized::Catch => GameMode::Catch,
        GameMode_Serialized::Mania => GameMode::Mania,
        _ => GameMode::Osu,
    }
}  

fn convert_to_path_type_to_normal(serialized: &PathType_Serialized) -> rosu_pp::parse::PathType {
    match serialized {
        PathType_Serialized::Catmull => rosu_pp::parse::PathType::Catmull,
        PathType_Serialized::Bezier => rosu_pp::parse::PathType::Bezier,
        PathType_Serialized::Linear => rosu_pp::parse::PathType::Linear,
        PathType_Serialized::PerfectCurve => rosu_pp::parse::PathType::PerfectCurve,
        _ => rosu_pp::parse::PathType::Linear,
    }
}
fn convert_to_path_control_point_to_normal(serialized: &PathControlPoint_Serialized) -> rosu_pp::parse::PathControlPoint {
    rosu_pp::parse::PathControlPoint {
        pos: Pos2{
            x: serialized.pos.x,
            y: serialized.pos.y,
        },// Convert Pos2_Serialized to Pos2
        kind: serialized.kind.as_ref().map(|k| convert_to_path_type_to_normal(k)),  // Convert Option<PathType_Serialized> to Option<PathType>
    }
}
pub fn hit_objects_to_normal(hit_objects: Vec<HitObject_Serialized>)->Vec<HitObject>{
    let mut new_vec = Vec::with_capacity(hit_objects.len());
    for item in hit_objects{
        new_vec.push(
            HitObject{
                pos: Pos2{
                    x: item.pos.x,
                    y: item.pos.y,
                },
                start_time: item.start_time,
                
                kind: match item.kind {
                    HitObjectKind_Serialized::Circle => rosu_pp::parse::HitObjectKind::Circle,
                    HitObjectKind_Serialized::Slider { pixel_len, repeats, control_points, edge_sounds } => {
                        rosu_pp::parse::HitObjectKind::Slider { 
                            pixel_len: pixel_len,
                            repeats: repeats,
                            control_points: control_points.into_iter().map(|p| convert_to_path_control_point_to_normal(&p)).collect(),
                            edge_sounds: edge_sounds,
                        }
                    },
                    HitObjectKind_Serialized::Spinner { end_time } => rosu_pp::parse::HitObjectKind::Spinner { end_time: end_time },
                    HitObjectKind_Serialized::Hold { end_time } => rosu_pp::parse::HitObjectKind::Hold { end_time: end_time },
                }
            }             
        )
    }
    new_vec
}
pub fn timing_points_to_normal(timing_points: Vec<TimingPoint_Serialized>)->SortedVec<rosu_pp::beatmap::TimingPoint>{
    let mut new_vec = SortedVec::<rosu_pp::beatmap::TimingPoint>::with_capacity(timing_points.len());
    for item in timing_points{
        new_vec.push(
            rosu_pp::beatmap::TimingPoint{
                beat_len: item.beat_len,
                time: item.time,
            }
        )
    }
    new_vec
}
pub fn difficulty_points_to_normal(difficulty_points: Vec<DifficultyPoint_Serialized>)->SortedVec<rosu_pp::beatmap::DifficultyPoint>{
    let mut new_vec = SortedVec::<rosu_pp::beatmap::DifficultyPoint>::with_capacity(difficulty_points.len());
    for item in difficulty_points{
        new_vec.push(
            rosu_pp::beatmap::DifficultyPoint{
                time: item.time,
                slider_vel: item.slider_vel,
                bpm_mult: item.bpm_mult,
                generate_ticks: item.generate_ticks,
            }
        )
    }
    new_vec
}
pub fn effect_points_to_normal(effect_points: Vec<EffectPoint_Serialized>)->SortedVec<rosu_pp::beatmap::EffectPoint>{
    let mut new_vec = SortedVec::<rosu_pp::beatmap::EffectPoint>::with_capacity(effect_points.len());
    for item in effect_points{
        new_vec.push(
            rosu_pp::beatmap::EffectPoint{
                time: item.time,
                kiai: item.kiai,
            }
        )
    }
    new_vec
}
pub fn serialized_to_normal_beatmap(serialized: Beatmap_Serialized)->Beatmap{
    Beatmap { 
        mode: match_gamemodes_to_normal(serialized.mode),
        version: serialized.version,
        n_circles: serialized.n_circles,
        n_sliders: serialized.n_sliders,
        n_spinners: serialized.n_spinners,
        ar: serialized.ar, 
        od: serialized.od,
        cs: serialized.cs,
        hp: serialized.hp,
        slider_mult: serialized.slider_mult,
        tick_rate: serialized.tick_rate,
        hit_objects: hit_objects_to_normal(serialized.hit_objects),
        sounds: serialized.sounds,
        timing_points: timing_points_to_normal(serialized.timing_points),
        difficulty_points: difficulty_points_to_normal(serialized.difficulty_points),
        effect_points: effect_points_to_normal(serialized.effect_points),
        stack_leniency: serialized.stack_leniency,
        breaks: serialized.breaks.into_iter().map(|b| rosu_pp::beatmap::Break{ start_time: b.start_time, end_time: b.end_time }).collect(), 
    }
}

/// ----------------------------- TO SERIALIZED --------------------------------------------------------------------------------------------------------------------------------------
pub fn normal_to_serialized_beatmap(normal: Beatmap)->Beatmap_Serialized{
    Beatmap_Serialized { 
        mode: match_gamemodes_to_serialized(normal.mode),
        version: normal.version,
        n_circles: normal.n_circles,
        n_sliders: normal.n_sliders,
        n_spinners: normal.n_spinners,
        ar: normal.ar, 
        od: normal.od,
        cs: normal.cs,
        hp: normal.hp,
        slider_mult: normal.slider_mult,
        tick_rate: normal.tick_rate,
        hit_objects: hit_objects_to_serialized(normal.hit_objects),
        sounds: normal.sounds,
        timing_points: timing_points_to_serialized(normal.timing_points),
        difficulty_points: difficulty_points_to_serialized(normal.difficulty_points),
        effect_points: effect_points_to_serialized(normal.effect_points),
        stack_leniency: normal.stack_leniency,
        breaks: normal.breaks.into_iter().map(|b| Break_Serialized{ start_time: b.start_time, end_time: b.end_time }).collect(), 
    }
}

#[inline(always)]
pub fn match_gamemodes_to_serialized(gamemode: GameMode)->GameMode_Serialized{
    match gamemode {
        GameMode::Osu => GameMode_Serialized::Osu,
        GameMode::Taiko => GameMode_Serialized::Taiko,
        GameMode::Catch => GameMode_Serialized::Catch,
        GameMode::Mania => GameMode_Serialized::Mania,
        _ => GameMode_Serialized::Osu,
    }
} 

fn convert_to_path_type_to_serialized(normal: &rosu_pp::parse::PathType) -> PathType_Serialized {
    match normal {
        rosu_pp::parse::PathType::Catmull => PathType_Serialized::Catmull,
        rosu_pp::parse::PathType::Bezier => PathType_Serialized::Bezier,
        rosu_pp::parse::PathType::Linear => PathType_Serialized::Linear,
        rosu_pp::parse::PathType::PerfectCurve => PathType_Serialized::PerfectCurve,
        _ => PathType_Serialized::Linear,
    }
}

fn convert_to_path_control_point_to_serialized(normal: &rosu_pp::parse::PathControlPoint) -> PathControlPoint_Serialized {
    PathControlPoint_Serialized {
        pos: Pos2_Serialized{
            x: normal.pos.x,
            y: normal.pos.y,
        },
        kind: normal.kind.as_ref().map(|k| convert_to_path_type_to_serialized(k)),
    }
}

pub fn hit_objects_to_serialized(hit_objects: Vec<HitObject>)->Vec<HitObject_Serialized>{
    let mut new_vec = Vec::with_capacity(hit_objects.len());
    for item in hit_objects{
        new_vec.push(
            HitObject_Serialized{
                pos: Pos2_Serialized{
                    x: item.pos.x,
                    y: item.pos.y,
                },
                start_time: item.start_time,

                kind: match item.kind {
                    rosu_pp::parse::HitObjectKind::Circle => HitObjectKind_Serialized::Circle,
                    rosu_pp::parse::HitObjectKind::Slider { pixel_len, repeats, control_points, edge_sounds } => {
                        HitObjectKind_Serialized::Slider { 
                            pixel_len: pixel_len,
                            repeats: repeats,
                            control_points: control_points.into_iter().map(|p| convert_to_path_control_point_to_serialized(&p)).collect(),
                            edge_sounds: edge_sounds,
                        }
                    },
                    rosu_pp::parse::HitObjectKind::Spinner { end_time } => HitObjectKind_Serialized::Spinner { end_time: end_time },
                    rosu_pp::parse::HitObjectKind::Hold { end_time } => HitObjectKind_Serialized::Hold { end_time: end_time },
                }
            }             
        )
    }
    new_vec
}

pub fn timing_points_to_serialized(timing_points: SortedVec<rosu_pp::beatmap::TimingPoint>)->Vec<TimingPoint_Serialized>{
    let mut new_vec = Vec::<TimingPoint_Serialized>::with_capacity(timing_points.len());
    for item in timing_points.into_iter() {
        new_vec.push(
            TimingPoint_Serialized{
                beat_len: item.beat_len,
                time: item.time,
            }
        )
    }
    new_vec
}

pub fn difficulty_points_to_serialized(difficulty_points: SortedVec<rosu_pp::beatmap::DifficultyPoint>)->Vec<DifficultyPoint_Serialized>{
    let mut new_vec = Vec::<DifficultyPoint_Serialized>::with_capacity(difficulty_points.len());
    for item in difficulty_points.into_iter(){
        new_vec.push(
            DifficultyPoint_Serialized{
                time: item.time,
                slider_vel: item.slider_vel,
                bpm_mult: item.bpm_mult,
                generate_ticks: item.generate_ticks,
            }
        )
    }
    new_vec
}
pub fn effect_points_to_serialized(effect_points: SortedVec<rosu_pp::beatmap::EffectPoint>)->Vec<EffectPoint_Serialized>{
    let mut new_vec = Vec::<EffectPoint_Serialized>::with_capacity(effect_points.len());
    for item in effect_points.into_iter() {
        new_vec.push(
            EffectPoint_Serialized{
                time: item.time,
                kiai: item.kiai,
            }
        )
    }
    new_vec
}
