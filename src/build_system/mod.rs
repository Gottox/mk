mod cargo;
mod cmake;
mod make;
mod meson;

use std::fmt::Debug;
use std::path::{Path, PathBuf};

use crate::project::Project;
use crate::Result;

pub static BUILD_SYSTEMS: &[&dyn BuildSystem] =
    &[&meson::Meson, &cargo::Cargo, &cmake::CMake, &make::Make];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootIdentificationResult {
    IsRoot,
    NotRoot,
    MaybeRoot,
}

pub fn build_system_from_str(name: &str) -> Option<&'static dyn BuildSystem> {
    match name {
        "cargo" => Some(&cargo::Cargo),
        "cmake" => Some(&cmake::CMake),
        "make" => Some(&make::Make),
        "meson" => Some(&meson::Meson),
        _ => None,
    }
}

pub trait BuildSystem: Debug + Sync {
    fn is_project_root(&self, path: &Path) -> Result<RootIdentificationResult>;
    fn configure_marker(&self, project: &Project) -> Result<Option<PathBuf>>;
    fn configure_command(&self, project: &Project) -> Vec<String>;
    fn build_command(&self, project: &Project) -> Vec<String>;
}
