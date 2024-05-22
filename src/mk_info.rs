use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{Error, Result};
use serde::Deserialize;

pub static MKINFO_FILES: &[&str] = &[
    ".Mk",
    ".Mk.yaml",
    ".Mk.yml",
    ".github/mk",
    ".github/mk.yaml",
    ".github/mk.yml",
    ".github/Mk",
    ".github/Mk.yaml",
    ".github/Mk.yml",
    ".mk",
    ".mk.yaml",
    ".mk.yml",
    "Mk",
    "Mk.yaml",
    "Mk.yml",
];

#[derive(Debug, Deserialize, Default)]
pub struct MkInfo {
    pub image: Option<String>,
    pub default: Option<Vec<String>>,
    pub configure: Option<Vec<String>>,
    pub build_system: Option<String>,
    pub build_dir: Option<PathBuf>,
    pub env: Option<HashMap<String, String>>,
}

impl MkInfo {
    pub fn from_root_path(root_path: &Path) -> Result<Self> {
        let mut mkinfo_iter = MKINFO_FILES
            .iter()
            .map(|mkinfo| root_path.join(mkinfo))
            .filter(|p| p.exists());

        let mkinfo = if let Some(mkinfo) = mkinfo_iter.next() {
            mkinfo
        } else {
            return Ok(Self::default());
        };

        if let Some(other_mkinfo) = mkinfo_iter.next() {
            return Err(Error::ConflictingMk(mkinfo, other_mkinfo));
        }

        Self::from_path(&mkinfo)
    }

    pub fn from_path(path: &Path) -> Result<Self> {
        let reader =
            std::fs::File::open(path).map_err(|e| Error::Io(path.into(), e))?;
        serde_yaml::from_reader(reader)
            .map_err(|e| Error::SerdeYaml(path.into(), e))
    }
}
