//! This module contains the structs for the configuration files.

use std::{
    borrow::Cow,
    fmt::Display,
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Deserializer};
use serde_derive::Serialize;
use text_io::read;
use toml::value::Value;
use tracing::{error, warn};
use url::Url;

use crate::constants::{GLOBAL_TEMPLATE_DIRECTORY, TEMPLATE_FILENAME};

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
        // TODO: Very likely not the good thing to do...
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

#[derive(Debug, Deserialize)]
pub struct TemplateRepositoryEntry {
    pub name: String,
    pub repository: Url,
    pub description: String,
}

impl Display for TemplateRepositoryEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} -- {}: {}",
            self.repository.path().get(1..).unwrap(),
            self.name,
            self.description
        )
    }
}

#[derive(Debug)]
pub enum TemplateRepository {
    Url(Url),
    Path(PathBuf),
}

impl Display for TemplateRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path(path) => write!(f, "{}", path.to_string_lossy()),
            Self::Url(url) => write!(f, "{}", url),
        }
    }
}

impl TemplateRepository {
    fn deserialize<'de, D>(deserializer: D) -> Result<Option<Self>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = match Option::<&str>::deserialize(deserializer) {
            Ok(Some(value)) => value,
            Ok(None) => return Ok(None),
            Err(error) => return Err(error),
        };

        if let Ok(url) = value.parse::<Url>() {
            Ok(Some(Self::Url(url)))
        } else {
            Ok(Some(Self::Path(Path::new(value).to_path_buf())))
        }
    }

    pub async fn read(&self) -> Vec<TemplateRepositoryEntry> {
        match self {
            Self::Path(path) => {
                let file = match File::open(path) {
                    Ok(file) => file,
                    Err(_error) => {
                        warn!("Couldn't find file located in {}", path.to_string_lossy());

                        return Vec::new();
                    }
                };

                let reader = BufReader::new(file);

                match serde_json::from_reader(reader) {
                    Ok(entries) => entries,
                    Err(error) => {
                        warn!("Template repository's format is invalid: {}", error);

                        Vec::new()
                    }
                }
            }
            Self::Url(url) => {
                let response = match reqwest::get(url.as_str()).await {
                    Ok(response) => response,
                    Err(_) => {
                        warn!("Couldn't access remote template repository {}", url);

                        return Vec::new();
                    }
                };

                match response.json().await {
                    Ok(entries) => entries,
                    Err(error) => {
                        warn!("Template repository's format is invalid: {}", error);

                        Vec::new()
                    }
                }
            }
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
    /// Set of custom keys the user can set in their global configuration file
    pub custom_keys: Option<CustomKeys>,
    /// A path or url that points to a templates repository file,
    /// that is a json file listing all the available templates
    #[serde(default, deserialize_with = "TemplateRepository::deserialize")]
    pub templates_repository: Option<TemplateRepository>,
}

impl Config {
    /// Given a `Path`, read the .toml file there as a configuration file.
    pub fn from_path<P: AsRef<Path>>(config_path: P) -> Self {
        let mut config_file = match File::open(&config_path) {
            Ok(config_file) => config_file,
            Err(_) => {
                warn!(
                    "File {} not found, using default configuration",
                    config_path.as_ref().to_string_lossy()
                );

                return Self::default();
            }
        };

        let mut toml_str = String::new();

        if config_file.read_to_string(&mut toml_str).is_err() {
            warn!(
                "File {} couldn't be read",
                config_path.as_ref().to_string_lossy()
            );

            std::process::exit(1);
        };

        match toml::from_str(&toml_str) {
            Ok(config) => config,
            Err(error) => {
                warn!(
                    "File {} was not properly formatted: {}",
                    config_path.as_ref().to_string_lossy(),
                    error
                );

                std::process::exit(1);
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
