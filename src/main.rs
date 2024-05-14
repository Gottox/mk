use std::{fs, path::PathBuf};
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
    #[error("Missing Argument for {0}")]
    MissingArgument(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Opts {
    clean: bool,
    reconfigure: bool,
    cwd: PathBuf,
    build_dir: Option<PathBuf>,
    args: Vec<String>,
}

impl Opts {
    fn parse() -> Result<Self> {
        let mut cwd = PathBuf::from(".");
        let mut build_dir = None;
        let mut args = vec![];
        let mut clean = false;
        let mut reconfigure = false;

        let mut args_iter = std::env::args().skip(1);
        while let Some(arg) = args_iter.next() {
            match arg.as_str() {
                "-mc" => clean = true,
                "-mR" => reconfigure = true,
                "-mC" => {
                    cwd = args_iter
                        .next()
                        .ok_or(Error::MissingArgument("-mC"))?
                        .into()
                }
                "-mB" => {
                    build_dir = Some(
                        args_iter
                            .next()
                            .ok_or(Error::MissingArgument("-mB"))?
                            .into(),
                    )
                }
                _ => args.push(arg),
            }
        }

        Ok(Self {
            clean,
            reconfigure,
            cwd,
            build_dir,
            args,
        })
    }
}

fn try_main() -> Result<()> {
    let opts = Opts::parse()?;
    let project = Project::from_opts(&opts)?;

    if opts.clean {
        return Ok(fs::remove_dir_all(&project.build_dir)?);
    }

    if opts.reconfigure || !project.is_configured()? {
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
