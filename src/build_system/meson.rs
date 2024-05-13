use std::fs::File;
use std::io::BufRead;

use crate::project::Project;

use super::{BuildSystem, RootIdentificationResult};

#[derive(Debug)]
pub struct Meson;

impl BuildSystem for Meson {
    fn is_project_root(
        &self,
        path: &std::path::Path,
    ) -> crate::Result<super::RootIdentificationResult> {
        let meson_path = path.join("meson.build");
        let meson_options_path = path.join("meson_options.txt");

        if meson_options_path.exists() {
            return Ok(RootIdentificationResult::IsRoot);
        }

        if !meson_path.exists() {
            return Ok(RootIdentificationResult::NotRoot);
        }

        let meson = File::open(meson_path)?;
        let meson = std::io::BufReader::new(meson);
        for line in meson.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            } else if line.starts_with("#") {
                continue;
            } else if line.starts_with("project") {
                return Ok(RootIdentificationResult::IsRoot);
            }
        }
        Ok(RootIdentificationResult::IsRoot)
    }

    fn is_configured(&self, project: &Project) -> crate::Result<bool> {
        Ok(project.build_dir.join("build.ninja").is_file())
    }

    fn configure_command(&self, project: &Project) -> Vec<String> {
        let mut command = vec!["meson".to_string(), "setup".to_string()];
        command.extend(project.configure_args.clone());
        command.extend([
            project.build_dir.to_string_lossy().to_string(),
            project.project_dir.to_string_lossy().to_string(),
        ]);
        command
    }

    fn build_command(&self, project: &Project) -> Vec<String> {
        let mut command = vec![
            "ninja".to_string(),
            "-C".to_string(),
            project.build_dir.to_string_lossy().to_string(),
        ];
        command.extend(project.args.clone());
        command
    }
}
