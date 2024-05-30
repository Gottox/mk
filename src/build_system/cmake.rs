use std::path::PathBuf;

use crate::{project::Project, Result};

use super::{BuildSystem, RootIdentificationResult};

#[derive(Debug)]
pub struct CMake;

impl BuildSystem for CMake {
    fn is_project_root(
        &self,
        path: &std::path::Path,
    ) -> Result<super::RootIdentificationResult> {
        use RootIdentificationResult::*;

        let cmakelists_path = path.join("CMakeLists.txt");
        if cmakelists_path.exists() {
            Ok(IsRoot)
        } else {
            Ok(NotRoot)
        }
    }
    fn configure_marker(&self, project: &Project) -> Result<Option<PathBuf>> {
        Ok(Some(project.build_dir.join("build.ninja")))
    }

    fn configure_command(&self, project: &Project) -> Vec<String> {
        let mut command = vec!["cmake".to_string()];
        command.extend(project.configure_args.clone());
        command.extend([
            "-G".to_string(),
            "Unix Makefiles".to_string(),
            "-S".to_string(),
            project.project_dir.to_string_lossy().to_string(),
            "-B".to_string(),
            project.build_dir.to_string_lossy().to_string(),
        ]);
        command
    }

    fn build_command(&self, project: &Project) -> Vec<String> {
        let mut command = vec![
            "make".to_string(),
            "-C".to_string(),
            project.build_dir.to_string_lossy().to_string(),
        ];
        command.extend(project.args.clone());
        command
    }
}
