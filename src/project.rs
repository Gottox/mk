use crate::{
    build_system::{RootIdentificationResult, BUILD_SYSTEMS},
    editor_config::EditorConfig,
    mk_info::MkInfo,
    Error, Result,
};
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{build_system::BuildSystem, Opts};

const VCS_ROOT_DIRS: &'static [&'static str] =
    &[".git", ".hg", "_darcs", ".bzr"];
const VCS_TIL_ROOT_DIRS: &'static [&'static str] = &[".svn", "CVS"];

fn has_root_editor_config(path: &Path) -> Result<bool> {
    path.join(".editorconfig")
        .exists()
        .then(|| EditorConfig::from_file(&path).map(|x| x.root))
        .unwrap_or(Ok(false))
}

fn is_project_root(path: &Path) -> Result<RootIdentificationResult> {
    use RootIdentificationResult::*;

    Ok(if VCS_ROOT_DIRS.iter().any(|x| path.join(x).is_dir()) {
        IsRoot
    } else if VCS_TIL_ROOT_DIRS.iter().any(|x| path.join(x).is_dir()) {
        MaybeRoot
    } else if has_root_editor_config(path)? {
        IsRoot
    } else {
        NotRoot
    })
}

pub fn find_root(path: &Path) -> Result<(&'static dyn BuildSystem, PathBuf)> {
    use RootIdentificationResult::*;
    let mut maybe_build_system = None;
    let mut was_maybe_root = false;

    for candidate in path.ancestors() {
        let last_maybe_build_system = maybe_build_system.take();
        for build_system in BUILD_SYSTEMS {
            match build_system.is_project_root(candidate)? {
                IsRoot => return Ok((*build_system, candidate.to_path_buf())),
                MaybeRoot => {
                    maybe_build_system =
                        Some((*build_system, candidate.to_path_buf()))
                }
                NotRoot => {}
            }
        }

        if maybe_build_system.is_none() && last_maybe_build_system.is_some() {
            break;
        }
        match is_project_root(candidate)? {
            IsRoot => break,
            MaybeRoot => was_maybe_root = true,
            NotRoot => {
                if was_maybe_root {
                    break;
                }
            }
        }
    }

    maybe_build_system.ok_or(Error::NoProjectRootFound)
}

#[derive(Debug)]
pub struct Project {
    pub project_dir: PathBuf,
    pub work_dir: PathBuf,
    pub build_dir: PathBuf,
    pub configure_args: Vec<String>,
    pub args: Vec<String>,
    pub build_system: &'static (dyn BuildSystem),
}

impl Project {
    pub fn from_opts(opts: Opts) -> Result<Self> {
        let work_dir = opts.mk_cwd.canonicalize()?;
        let (build_system, project_dir) = find_root(&work_dir)?;

        let mk_info = MkInfo::from_root_path(&project_dir)?;

        let configure_args = mk_info.configure.unwrap_or_default();
        let build_dir = opts
            .mk_build_dir
            .or(mk_info.build_dir)
            .unwrap_or_else(|| "build".into());
        let build_dir = project_dir.join(&build_dir);
        let args = opts.args.or(mk_info.default).unwrap_or_default();

        Ok(Self {
            project_dir,
            work_dir,
            build_dir,
            build_system,
            configure_args,
            args,
        })
    }

    pub fn run(&self, command: &[String]) -> Result<()> {
        Command::new(&command[0])
            .args(command.iter().skip(1))
            .current_dir(&self.work_dir)
            .status()?;
        Ok(())
    }
    pub fn build(&self) -> Result<()> {
        let cmd = self.build_system.build_command(&self);
        self.run(&cmd)
    }

    pub fn configure(&self) -> Result<()> {
        let cmd = self.build_system.configure_command(&self);
        if cmd.len() > 1 {
            self.run(&cmd)
        } else {
            Ok(())
        }
    }

    pub fn is_configured(&self) -> Result<bool> {
        self.build_system.is_configured(&self)
    }
}
