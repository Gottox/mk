use std::{fs::File, io::BufRead, path::PathBuf};

use crate::{project::Project, Error};

use super::{BuildSystem, RootIdentificationResult};

#[derive(Debug)]
pub struct Meson;

impl BuildSystem for Meson {
    fn is_project_root(
        &self,
        path: &std::path::Path,
    ) -> crate::Result<super::RootIdentificationResult> {
        use RootIdentificationResult::*;

        let meson_options_path = path.join("meson_options.txt");
        if meson_options_path.exists() {
            return Ok(IsRoot);
        }

        let meson_path = path.join("meson.build");
        if !meson_path.exists() {
            return Ok(NotRoot);
        }
        let meson = File::open(&meson_path)
            .map_err(|e| Error::Io(meson_path.clone(), e))?;
        let meson = std::io::BufReader::new(meson);
        for line in meson.lines() {
            let line = line.map_err(|e| Error::Io(meson_path.clone(), e))?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            } else if line.starts_with("project") {
                return Ok(IsRoot);
            }
        }
        Ok(MaybeRoot)
    }

    fn configure_marker(
        &self,
        project: &Project,
    ) -> crate::Result<Option<PathBuf>> {
        Ok(Some(project.build_dir.join("build.ninja")))
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
