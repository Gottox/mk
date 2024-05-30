use std::path::PathBuf;

use crate::project::Project;
use crate::Result;

use super::BuildSystem;
use super::RootIdentificationResult;

#[derive(Debug)]
pub struct Cargo;

impl BuildSystem for Cargo {
    fn is_project_root(
        &self,
        path: &std::path::Path,
    ) -> Result<RootIdentificationResult> {
        use RootIdentificationResult::*;

        let cargo_toml_path = path.join("Cargo.toml");
        Ok(if cargo_toml_path.is_file() {
            MaybeRoot
        } else {
            NotRoot
        })
    }
    fn configure_marker(&self, _project: &Project) -> Result<Option<PathBuf>> {
        Ok(None)
    }

    fn configure_command(&self, _project: &Project) -> Vec<String> {
        vec![]
    }

    fn build_command(&self, project: &Project) -> Vec<String> {
        let mut command = vec!["cargo".to_string()];
        if project.args.is_empty() {
            command.push("build".to_string());
        } else {
            command.extend(project.args.clone());
        }
        command
    }
}
