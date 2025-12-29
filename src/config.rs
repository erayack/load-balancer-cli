use std::fs;
use std::path::Path;

use crate::models::{SimConfig, SimError, SimResult};

pub fn load_config(path: &Path) -> SimResult<SimConfig> {
    let contents = fs::read_to_string(path).map_err(|err| {
        SimError::ConfigIo(format!(
            "failed to read config '{}': {}",
            path.display(),
            err
        ))
    })?;
    let ext = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");

    match ext {
        "toml" => toml::from_str(&contents)
            .map_err(|err| SimError::ConfigParse(format!("failed to parse TOML: {}", err))),
        "json" => serde_json::from_str(&contents)
            .map_err(|err| SimError::ConfigParse(format!("failed to parse JSON: {}", err))),
        "" => Err(SimError::UnsupportedConfigFormat("unknown".to_string())),
        _ => Err(SimError::UnsupportedConfigFormat(ext.to_string())),
    }
}
