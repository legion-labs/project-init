//! This module contains the structs for the configuration files.

use std::{
    borrow::Cow,
    fmt::Display,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use serde_derive::{Deserialize, Serialize};
use text_io::read;
use toml::value::Value;
use tracing::{error, warn};

const TEMPLATE_FILENAME: &str = "template.toml";

const GLOBAL_TEMPLATE_DIRECTORY: &str = ".pi_templates";

/// Struct for the author. This is read from the global
/// configuration that resides at $HOME/.pi.toml
#[derive(Debug, Deserialize)]
pub struct Author {
    pub name: String,
    pub email: String,
    pub github_username: Option<String>,
}

impl Author {
    pub fn new<'a, N, E>(name: N, email: E) -> Self
    where
        N: Into<Cow<'a, str>>,
        E: Into<Cow<'a, str>>,
    {
        Self {
            name: name.into().into_owned(),
            email: email.into().into_owned(),
            github_username: None,
        }
    }

    /// Reads terminal input to ask for the name and email to the user
    pub fn read_input() -> Self {
        println!("Enter your name");

        let name: String = read!("{}");

        println!("Enter your email");

        let email: String = read!("{}");

        Author::new(name, email)
    }
}

impl Default for Author {
    fn default() -> Self {
        Author::read_input()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionControl {
    Git,
    Hg,
    Mercurial,
    Pijul,
    Darcs,
    #[serde(other)]
    Unknown,
}

impl Display for VersionControl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionControl::Git => write!(f, "git"),
            VersionControl::Hg => write!(f, "hg"),
            VersionControl::Mercurial => write!(f, "mercurial"),
            VersionControl::Pijul => write!(f, "pijul"),
            VersionControl::Darcs => write!(f, "darcs"),
            VersionControl::Unknown => write!(f, "Unknown Version Control"),
        }
    }
}

/// Struct for the global configuration at $HOME/.pi.toml
#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub version_control: Option<VersionControl>,
    #[serde(default)]
    pub author: Author,
    pub license: Option<License>,
    pub custom_keys: Option<CustomKeys>,
}

impl Config {
    /// Given a `Path`, read the .toml file there as a configuration file.
    pub fn from_path<P: AsRef<Path>>(config_path: P) -> Self {
        let mut config_file = match File::open(&config_path) {
            Ok(config_file) => config_file,
            Err(_) => {
                warn!("File ~/.pi.toml not found, using default configuration");

                return Self::default();
            }
        };

        let mut toml_str = String::new();

        if config_file.read_to_string(&mut toml_str).is_err() {
            warn!("File ~/.pi.toml couldn't be read");

            return Self::default();
        };

        match toml::from_str(&toml_str) {
            Ok(config) => config,
            Err(_error) => {
                warn!("File ~/.pi.toml was not properly formatted, using default configuration");

                Self::default()
            }
        }
    }
}

/// Struct for directories, files, templates, and scripts to be created.
#[derive(Debug, Deserialize)]
pub struct Directory {
    pub files: Option<Vec<PathBuf>>,
    pub directories: Option<Vec<PathBuf>>,
    pub templates: Option<Vec<PathBuf>>,
    pub scripts: Option<Vec<PathBuf>>,
}

/// Struct for project-specific configuration options
#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub version_control: Option<VersionControl>,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum License {
    Bsd3,
    Bsd,
    Gpl3,
    Mit,
    AllRightsReserved,
    #[serde(other)]
    Unknown,
}

impl Display for License {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            License::Bsd3 => write!(f, "BSD3"),
            License::Bsd => write!(f, "BSD"),
            License::Gpl3 => write!(f, "GPL3"),
            License::Mit => write!(f, "MIT"),
            License::AllRightsReserved => write!(f, "All Rights Reserved"),
            License::Unknown => write!(f, "Unknown License"),
        }
    }
}

/// Struct for a project
#[derive(Debug, Deserialize)]
pub struct Project {
    pub license: Option<License>,
    #[serde(default)]
    pub with_readme: bool,
    // TODO: Rename to directories, or rename `Directory` to `File`?
    pub files: Directory,
    pub config: Option<ProjectConfig>,
    pub custom_keys: Option<CustomKeys>,
    // Set manually
    #[serde(skip)]
    pub path: PathBuf,
}

impl Project {
    /// Given a filepath, read the .toml file there as containing the
    /// directories/templates.
    /// If no such file is found, read from global template directory in
    /// `$HOME/.pi_templates/`.
    pub fn from_path<D: AsRef<Path>, H: AsRef<Path>>(home: H, directory: D) -> Self {
        let template_path = directory.as_ref().join(TEMPLATE_FILENAME);

        let (mut template_file, path) = match File::open(&template_path) {
            Ok(file) => (file, directory.as_ref().to_path_buf()),
            Err(_) => {
                let global_directory = home
                    .as_ref()
                    .join(GLOBAL_TEMPLATE_DIRECTORY)
                    .join(&directory);

                let global_template_path = global_directory.join(TEMPLATE_FILENAME);

                match File::open(&global_template_path) {
                    Ok(file) => (file, global_directory),
                    Err(_) => {
                        error!(
                            "File {:?} could not be opened, does it exist?",
                            global_template_path
                        );

                        std::process::exit(0x0f00);
                    }
                }
            }
        };

        let mut template = String::new();

        if template_file.read_to_string(&mut template).is_err() {
            error!("Couldn't read content of file {:?}", path);

            std::process::exit(0x0f00);
        }

        let mut project: Self = match toml::from_str(&template) {
            Ok(project) => project,
            Err(error) => {
                error!("Error parsing {:?}: {}", directory.as_ref(), error);

                std::process::exit(0x0f00);
            }
        };

        project.path = path;

        project
    }
}

/// Struct for custom user keys
#[derive(Debug, Deserialize)]
pub struct CustomKeys {
    pub toml: Value,
}
