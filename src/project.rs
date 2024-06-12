use crate::{
    build_system::{
        build_system_from_str, RootIdentificationResult, BUILD_SYSTEMS,
    },
    editor_config::EditorConfig,
    mk_info::MkInfo,
    Error, Result,
};
use std::{
    collections::HashMap,
    env, io, iter,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    time::SystemTime,
};

use crate::{build_system::BuildSystem, Opts};

const VCS_ROOT_DIRS: &[&str] = &[".git", ".hg", "_darcs", ".bzr"];
const VCS_TIL_ROOT_DIRS: &[&str] = &[".svn", "CVS"];

fn has_root_editor_config(path: &Path) -> Result<bool> {
    path.join(".editorconfig")
        .exists()
        .then(|| EditorConfig::from_file(path).map(|x| x.root))
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

pub struct RootInfo {
    build_system: Option<&'static dyn BuildSystem>,
    project_dir: PathBuf,
}

impl RootInfo {
    fn new(
        build_system: Option<&'static dyn BuildSystem>,
        project_dir: &Path,
    ) -> Self {
        Self {
            build_system,
            project_dir: project_dir.to_path_buf(),
        }
    }
}

pub fn find_root(path: &Path) -> Result<RootInfo> {
    use RootIdentificationResult::*;
    let mut maybe_build_system = None;
    let mut maybe_root = false;

    for candidate in path.ancestors() {
        let last_maybe_build_system = maybe_build_system.take();
        for build_system in BUILD_SYSTEMS {
            match build_system.is_project_root(candidate)? {
                IsRoot => {
                    return Ok(RootInfo::new(Some(*build_system), candidate))
                }
                MaybeRoot => {
                    maybe_build_system =
                        Some(RootInfo::new(Some(*build_system), candidate));
                }
                NotRoot => {}
            }
        }

        if maybe_build_system.is_none() && last_maybe_build_system.is_some() {
            maybe_build_system = last_maybe_build_system;
            break;
        }
        match is_project_root(candidate)? {
            IsRoot => break,
            MaybeRoot => {
                maybe_root = true;
                if maybe_build_system.is_none() {
                    maybe_build_system = Some(RootInfo::new(None, candidate))
                }
            }
            NotRoot => {
                if maybe_root {
                    break;
                }
            }
        }
    }

    maybe_build_system.ok_or(Error::NoProjectRootFound)
}

pub struct Project {
    pub container_image: Option<String>,
    pub container_args: Option<Vec<String>>,
    pub mk_info_path: Option<PathBuf>,
    pub project_dir: PathBuf,
    pub work_dir: PathBuf,
    pub build_dir: PathBuf,
    pub configure_args: Vec<String>,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub build_system: &'static (dyn BuildSystem),
    pub container: bool,
}

impl Project {
    pub fn from_opts(opts: &Opts) -> Result<Self> {
        let work_dir = opts
            .cwd
            .canonicalize()
            .map_err(|e| Error::Io(opts.cwd.clone(), e))?;
        let RootInfo {
            build_system,
            project_dir,
        } = find_root(&work_dir)?;

        let mode = if let Ok(mode) = env::var("MKMODE") {
            mode
        } else {
            "default".to_string()
        };
        let mk_info_path = if let Ok(mk_info) = env::var("MKINFO") {
            Some(PathBuf::from(mk_info))
        } else {
            MkInfo::find_root_path(&project_dir)?
        };

        let mk_info = if let Some(mk_info_path) = &mk_info_path {
            MkInfo::from_path(mk_info_path)?
        } else {
            MkInfo::default()
        };

        let mut build_info = mk_info.base;
        if let Some(mode_info) = mk_info.mode {
            for mode in mode.split_whitespace() {
                build_info = build_info.merge(mode_info.get(mode).cloned());
            }
        }

        let configure_args = build_info.configure.clone().unwrap_or_default();
        let build_dir = opts.build_dir.clone().unwrap_or("build".into());
        let build_dir = project_dir.join(build_dir);
        let container = opts.container;
        let container_image = build_info.image().map(|x| x.to_string());
        let container_args = build_info.container_args().map(|x| x.to_vec());
        let args = if opts.args.is_empty() {
            build_info.default.unwrap_or_default().into()
        } else {
            opts.args.clone()
        };

        let build_system =
            if let Some(build_system) = build_info.build_system {
                build_system_from_str(&build_system)
            } else {
                build_system
            }
            .ok_or(Error::NoBuildSystemFound)?;

        let env = build_info.env.unwrap_or_default();

        Ok(Self {
            container,
            container_image,
            container_args,
            mk_info_path,
            project_dir,
            work_dir,
            build_dir,
            build_system,
            configure_args,
            args,
            env,
        })
    }

    pub fn clean(&self) -> Result<()> {
        match std::fs::remove_dir_all(&self.build_dir) {
            Ok(()) => Ok(()),
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(Error::Io(self.build_dir.clone(), e))
                }
            }
        }
    }

    pub fn find_container_runtime(&self) -> Result<PathBuf> {
        if let Ok(runtime) = std::env::var("CONTAINER_RUNTIME") {
            return Ok(PathBuf::from(runtime));
        }
        let runtimes = ["podman", "docker"];
        let Ok(path_var) = std::env::var("PATH") else {
            return Err(Error::NoContainerRuntimeFound);
        };

        for path in path_var.split(':') {
            for rt in &runtimes {
                let path = Path::new(path).join(rt);
                if path.exists() {
                    return Ok(path);
                }
            }
        }
        Err(Error::NoContainerRuntimeFound)
    }

    pub fn run(&self, command: &[String]) -> Result<ExitStatus> {
        let command = if self.container {
            let container_image = self
                .container_image
                .clone()
                .ok_or(Error::MissingContainerImage)?;
            let container_runtime = self.find_container_runtime()?;

            iter::once(container_runtime.to_string_lossy().to_string())
                .chain([
                    "run".to_string(),
                    "-ti".to_string(),
                    "--rm".to_string(),
                    "-v".to_string(),
                    format!("{0}:{0}", self.project_dir.display()),
                    "--workdir".to_string(),
                    self.work_dir.display().to_string(),
                ])
                .chain(self.env.iter().map(|(k, v)| format!("-e{}={}", k, v)))
                .chain(self.container_args.clone().unwrap_or_default())
                .chain(["--".to_string(), container_image])
                .chain(command.iter().cloned())
                .collect::<Vec<String>>()
        } else {
            command.to_vec()
        };

        Command::new(&command[0])
            .args(command.iter().skip(1))
            .envs(&self.env)
            .current_dir(&self.work_dir)
            .status()
            .map_err(|e| Error::Command(command[0].clone(), e))
    }
    pub fn build(&self) -> Result<ExitStatus> {
        let cmd = self.build_system.build_command(self);
        self.run(&cmd)
    }

    pub fn configure(&self) -> Result<ExitStatus> {
        let cmd = self.build_system.configure_command(self);
        if cmd.len() > 1 {
            self.run(&cmd)
        } else {
            Ok(ExitStatus::default())
        }
    }

    fn get_mtime(path: &Path) -> Result<SystemTime> {
        path.metadata()
            .and_then(|x| x.modified())
            .map_err(|e| Error::Io(path.into(), e))
    }

    pub fn is_configured(&self) -> Result<bool> {
        let marker =
            if let Some(marker) = self.build_system.configure_marker(self)? {
                marker
            } else {
                return Ok(true);
            };

        if !marker.exists() {
            return Ok(false);
        }

        let marker_time = Self::get_mtime(&marker)?;

        let mk_info_time = if let Some(mk_info_path) = &self.mk_info_path {
            Self::get_mtime(mk_info_path)?
        } else {
            return Ok(true);
        };

        Ok(marker_time > mk_info_time)
    }
}
