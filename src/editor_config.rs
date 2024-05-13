use crate::Result;
use std::path::Path;

use serde::Deserialize;

#[derive(Deserialize, Debug)]

pub struct EditorConfig {
    #[serde(default)]
    pub root: bool,
}

impl EditorConfig {
    pub fn from_file(path: &Path) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let config = serde_ini::from_read(reader)?;
        Ok(config)
    }
}
