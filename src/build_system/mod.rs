mod cargo;
mod cmake;
mod make;
mod meson;

use std::fmt::Debug;
use std::path::Path;

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

pub trait BuildSystem: Debug + Sync {
    fn is_project_root(&self, path: &Path) -> Result<RootIdentificationResult>;
    fn is_configured(&self, project: &Project) -> Result<bool>;
    fn configure_command(&self, project: &Project) -> Vec<String>;
    fn build_command(&self, project: &Project) -> Vec<String>;
}
