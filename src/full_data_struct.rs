use std::str::FromStr;

use serde::{Deserialize,Serialize};

use strum_macros::EnumIter;



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
            FullDataEnum::LongestStream => self.longest_stream.to_string(),
            FullDataEnum::Streams100 => self.streams100.to_string(),
            FullDataEnum::AvgJumpsDistance => self.avg_jump_distance.to_string(),
            FullDataEnum::AvgJumpsSpeed => format!("{:.2}", self.avg_jump_speed),
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
            "MD5"=>Err(()),//Always added in the end :)        ---Ok(FullDataEnum::MD5),

             _=>Err(()),

        }
    }
}
