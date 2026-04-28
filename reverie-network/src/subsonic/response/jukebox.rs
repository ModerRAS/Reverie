//! Jukebox response DTOs

use reverie_core::JukeboxStatus;
use serde::Serialize;

/// Jukebox status response wrapper
#[derive(Debug, Clone, Serialize)]
pub struct JukeboxStatusData {
    #[serde(rename = "jukeboxStatus")]
    pub jukebox_status: JukeboxStatusItem,
}

#[derive(Debug, Clone, Serialize)]
pub struct JukeboxStatusItem {
    #[serde(rename = "currentIndex")]
    pub current_index: i32,
    pub playing: bool,
    #[serde(skip_serializing_if = "skip_float")]
    pub gain: f32,
    pub position: i32,
    pub volume: i32,
}

fn skip_float(v: &f32) -> bool {
    v.abs() < 0.001
}

impl From<&JukeboxStatus> for JukeboxStatusItem {
    fn from(s: &JukeboxStatus) -> Self {
        Self {
            current_index: s.current_index,
            playing: s.playing,
            gain: s.gain,
            position: s.position,
            volume: s.volume,
        }
    }
}

impl From<JukeboxStatusData> for super::ResponseData {
    fn from(v: JukeboxStatusData) -> Self {
        super::ResponseData::JukeboxStatus(v)
    }
}
