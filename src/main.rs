use std::{fs, path::PathBuf};

use clap::Parser;

pub mod build_system;
pub mod editor_config;
pub mod mk_info;
pub mod project;

use project::Project;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("No build system found")]
    NoBuildSystemFound,
    #[error("No project root found")]
    NoProjectRootFound,
    #[error("Conflicting Mk files: {0} and {1}")]
    ConflictingMk(PathBuf, PathBuf),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}: {1}")]
    SerdeIni(PathBuf, serde_ini::de::Error),
    #[error("{0}: {1}")]
    SerdeYaml(PathBuf, serde_yaml::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Parser, Debug)]
#[command(ignore_errors = true)]
pub struct Opts {
    #[arg(long, env, default_value = ".")]
    mk_cwd: PathBuf,

    #[arg(long, env)]
    mk_build_dir: Option<PathBuf>,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Option<Vec<String>>,
}

fn try_main() -> Result<()> {
    let opts = Opts::parse();
    let clean = opts
        .args
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .any(|arg| arg == "mk-clean");
    let project = Project::from_opts(opts)?;

    if clean {
        return Ok(fs::remove_dir_all(&project.build_dir)?);
    }

    if project.is_configured()? == false {
        project.configure()?;
    }

    project.build()
}

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
