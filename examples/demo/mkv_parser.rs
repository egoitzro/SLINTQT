use serde::Deserialize;
use std::process::Command;

#[derive(Deserialize, Debug)]
pub struct MkvTrackProperties {
    pub language: Option<String>,
    pub track_name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct MkvTrack {
    pub id: u32,
    #[serde(rename = "type")]
    pub track_type: String,
    pub codec: String,
    pub properties: Option<MkvTrackProperties>,
}

#[derive(Deserialize, Debug)]
pub struct MkvIdentifyResult {
    pub tracks: Vec<MkvTrack>,
}

pub fn analyze_file(file_path: &str) -> Result<Vec<MkvTrack>, String> {
    let mut mkvmerge_cmd = "mkvmerge";
    
    // Fallback if mkvmerge is not in PATH (Standard Windows MKVToolNix install path)
    let fallback_path = "C:\\Program Files\\MKVToolNix\\mkvmerge.exe";
    if std::path::Path::new(fallback_path).exists() {
        mkvmerge_cmd = fallback_path;
    }

    let output = Command::new(mkvmerge_cmd)
        .args(["--identify", "--identification-format", "json", file_path])
        .output()
        .map_err(|e| format!("Failed to run mkvmerge (¿está instalado en PATH o en C:\\Program Files\\MKVToolNix\\?): {}", e))?;

    if !output.status.success() {
        return Err(format!("mkvmerge failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let json_output = String::from_utf8_lossy(&output.stdout);
    let result: MkvIdentifyResult = serde_json::from_str(&json_output)
        .map_err(|e| format!("Failed to parse mkvmerge output: {}", e))?;

    Ok(result.tracks)
}
