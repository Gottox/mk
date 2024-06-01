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

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ContainerDef {
    Image(String),
    Definition {
        image: String,
        opts: Option<Vec<String>>,
    },
}

#[derive(Debug, Deserialize, Default)]
pub struct MkInfo {
    pub container: Option<ContainerDef>,
    pub default: Option<Vec<String>>,
    pub configure: Option<Vec<String>>,
    pub build_system: Option<String>,
    pub env: Option<HashMap<String, String>>,
}

impl MkInfo {
    pub fn find_root_path(root_path: &Path) -> Result<Option<PathBuf>> {
        let mut mkinfo_iter = MKINFO_FILES
            .iter()
            .map(|mkinfo| root_path.join(mkinfo))
            .filter(|p| p.exists());

        let path = if let Some(mkinfo) = mkinfo_iter.next() {
            mkinfo
        } else {
            return Ok(None);
        };

        if let Some(other_mkinfo) = mkinfo_iter.next() {
            return Err(Error::ConflictingMk(path, other_mkinfo));
        }

        Ok(Some(path))
    }
    pub fn from_root_path(root_path: &Path) -> Result<Self> {
        Self::from_path(&Self::find_root_path(root_path)?.unwrap_or_default())
    }

    pub fn from_path(path: &Path) -> Result<Self> {
        let reader =
            std::fs::File::open(path).map_err(|e| Error::Io(path.into(), e))?;
        serde_yaml::from_reader(reader)
            .map_err(|e| Error::SerdeYaml(path.into(), e))
    }

    pub fn image(&self) -> Option<&str> {
        match &self.container {
            Some(ContainerDef::Image(image)) => Some(image),
            Some(ContainerDef::Definition { image, .. }) => Some(image),
            None => None,
        }
    }

    pub fn container_args(&self) -> Option<&[String]> {
        match &self.container {
            Some(ContainerDef::Definition { opts, .. }) => opts.as_deref(),
            _ => None,
        }
    }
}
