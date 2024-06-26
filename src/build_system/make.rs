use std::path::PathBuf;

use crate::project::Project;
use crate::Result;

use super::BuildSystem;
use super::RootIdentificationResult;

#[derive(Debug)]
pub struct Make;

impl BuildSystem for Make {
    fn is_project_root(
        &self,
        path: &std::path::Path,
    ) -> Result<RootIdentificationResult> {
        use RootIdentificationResult::*;

        let makefile_path = path.join("Makefile");
        Ok(if makefile_path.is_file() {
            MaybeRoot
        } else {
            NotRoot
        })
    }

    fn configure_marker(
        &self,
        _project: &Project,
    ) -> crate::Result<Option<PathBuf>> {
        Ok(None)
    }

    fn configure_command(&self, _project: &Project) -> Vec<String> {
        vec![]
    }

    fn build_command(&self, project: &Project) -> Vec<String> {
        let mut command = vec![
            "make".to_string(),
            "-C".to_string(),
            project.project_dir.to_string_lossy().to_string(),
        ];
        command.extend(project.args.clone());
        command
    }
}
