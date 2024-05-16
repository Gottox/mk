use std::{
    fs,
    path::PathBuf,
    thread::sleep,
    time::{Duration, SystemTime},
};
pub mod build_system;
pub mod editor_config;
pub mod mk_info;
pub mod project;

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use project::Project;
use thiserror::Error;

static HELP: &str = r#"Usage: mk [options] [build system args]

Options:
    -mw: Watch for changes and rebuild
    -mc: Clean the build directory
    -mR: Force reconfigure
    -mC <dir>: Change the current working directory [default: .]
    -mB <dir>: Change the build directory [default: build]

Supported build systems:
meson/ninja
cmake/make
make
cargo
"#;

#[derive(Error, Debug)]
pub enum Error {
    #[error("No build system found")]
    NoBuildSystemFound,
    #[error("No project root found")]
    NoProjectRootFound,
    #[error("Conflicting Mk files: {0} and {1}")]
    ConflictingMk(PathBuf, PathBuf),
    #[error("{0}: {1}")]
    Io(PathBuf, std::io::Error),
    #[error("{0}: {1}")]
    Command(String, std::io::Error),
    #[error("{0}: {1}")]
    SerdeIni(PathBuf, serde_ini::de::Error),
    #[error("{0}: {1}")]
    SerdeYaml(PathBuf, serde_yaml::Error),
    #[error("{0}")]
    Notify(#[from] notify::Error),
    #[error("Missing Argument for {0}")]
    MissingArgument(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Opts {
    args: Vec<String>,
    build_dir: Option<PathBuf>,
    clean: bool,
    cwd: PathBuf,
    reconfigure: bool,
    watch: bool,
}

impl Opts {
    fn parse() -> Result<Self> {
        let mut cwd = PathBuf::from(".");
        let mut build_dir = None;
        let mut args = vec![];
        let mut clean = false;
        let mut reconfigure = false;
        let mut watch = false;

        let mut args_iter = std::env::args().skip(1);
        while let Some(arg) = args_iter.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    println!("{}", HELP);
                    std::process::exit(0);
                }
                "-mw" => watch = true,
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
                "--" => {
                    args.extend(args_iter);
                    break;
                }
                _ => args.push(arg),
            }
        }

        Ok(Self {
            args,
            build_dir,
            clean,
            cwd,
            reconfigure,
            watch,
        })
    }
}

fn try_main() -> Result<()> {
    let opts = Opts::parse()?;
    let project = Project::from_opts(&opts)?;

    if opts.clean {
        return fs::remove_dir_all(&project.build_dir)
            .map_err(|e| Error::Io(project.build_dir, e));
    }

    if opts.reconfigure || !project.is_configured()? {
        project.configure()?;
    }

    project.build()?;

    if opts.watch {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
        watcher.watch(&project.project_dir, RecursiveMode::Recursive)?;

        let threshold = Duration::from_millis(100);
        let mut last_build_time = SystemTime::now();
        for _ in rx {
            if last_build_time + threshold < SystemTime::now() {
                sleep(threshold);
                project.build()?;
                last_build_time = SystemTime::now();
            }
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
