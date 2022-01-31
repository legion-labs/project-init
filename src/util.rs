//! This library provides the functions/structs/methods used by the main
//! binary. They are included
//! here in the hopes that they can be illuminating to users.

use std::fs;
use std::path::Path;

use case::*;
use chrono::{Datelike, Utc};
use heck::ToUpperCamelCase;
use rustache::{HashBuilder, VecBuilder};
use toml::Value::Table;
use tracing::{error, warn};

use crate::includes;
use crate::render::{render_dirs, render_file, render_files, render_templates};
use crate::repo::{darcs_init, git_init, hg_init, pijul_init};
use crate::types::{Author, Config, License, Project, ProjectConfig, VersionControl};

/// Main orchestrator function.
///
/// Takes the name (which is also for the moment the output dir) of the project,
/// the global [`Config`] struct (as parsed from the `$HOME/.pi.toml` file),
/// the [`Project`] struct (as parsed from the project's `template.toml` file),
/// and a `force` argument.
///
/// It will automatically call the proper render functions, create the required
/// files and directories and populate them.
pub fn init_helper(
    name: &str,
    config: Config,
    project: Project,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let now = Utc::now();

    let year = now.year();

    let formatted_date = format!(
        "{month}-{day}-{year}",
        month = now.month0(),
        day = now.day0(),
        year = year
    );

    let project_files = project.files;

    let project_config = project.config;

    // prefer project-specific license over global
    let license = project.license.or(config.license);

    // set license if it's set
    let license_contents =
        // prefer project-specific license over global
        match license {
            None => {
                warn!("Requested license not specified, license file not generated");

                None
            },
            Some(License::Unknown) => {
                warn!("Unknown requested license, license file not generated");

                None
            }
            Some(License::Bsd3) => Some(includes::BSD3),
            Some(License::Bsd) => Some(includes::BSD),
            Some(License::Mit) => Some(includes::MIT),
            Some(License::Gpl3) => Some(includes::GPL3),
            Some(License::AllRightsReserved) => Some(includes::ALL_RIGHTS_RESERVED),
        };

    // set version
    let version = match project_config {
        Some(ProjectConfig {
            version: Some(ref version),
            ..
        }) => version.to_string(),
        _ => {
            warn!("No version info found, defaulting to '0.1.0'");

            "0.1.0".to_string()
        }
    };

    // set github username to null if it's not provided
    let github_username = match config.author {
        Some(Author {
            github_username: Some(ref github_username),
            ..
        }) => github_username,
        _ => {
            warn!("No github username found, defaulting to ''");

            ""
        }
    };

    // make custom_keys into a vector; prepare to insert them into the `HashBuilder`
    let custom_keys =
        if let Some(Table(custom_keys)) = project.custom_keys.map(|custom_keys| custom_keys.toml) {
            Some(custom_keys)
        } else {
            None
        };

    // make custom_keys into a vector; prepare to insert them into the `HashBuilder`
    let custom_keys_global = if let Some(Table(custom_keys_global)) =
        config.custom_keys.map(|custom_keys| custom_keys.toml)
    {
        Some(custom_keys_global)
    } else {
        None
    };

    // Make a hash for inserting stuff into templates.
    let mut keys = HashBuilder::new();

    // project-specific
    if let Some(custom_keys) = custom_keys {
        for (key, value) in &custom_keys {
            if let Some(value) = value.as_str() {
                keys = keys.insert(key, value);
            }
        }
    }

    // global
    if let Some(custom_keys) = custom_keys_global {
        for (key, value) in &custom_keys {
            if let Some(value) = value.as_str() {
                keys = keys.insert(key, value);
            }
        }
    }

    // add the normal stuff
    keys = keys
        .insert("project", name)
        .insert("Project", name.to_capitalized())
        .insert("ProjectCamelCase", name.to_upper_camel_case())
        .insert("year", year)
        .insert("version", version)
        .insert("github_username", github_username)
        .insert("date", formatted_date);

    match config.author {
        Some(Author { email, name, .. }) => {
            keys = keys.insert("name", name);
            keys = keys.insert("email", email);
        }
        _ => {
            keys = keys.insert("name", "");
            keys = keys.insert("email", "");
        }
    };

    if let Some(license) = license {
        keys = keys.insert("license", license.to_string())
    }

    // check if the directory exists and exit, if we haven't forced an overwrite.
    if Path::new(name).exists() && !force {
        error!(
            "Path '{}' already exists, rerun with -f or --force to overwrite",
            name
        );

        std::process::exit(0x0f00);
    };

    // create directories
    let _ = fs::create_dir(name);

    if let Some(directories) = project_files.directories {
        render_dirs(directories, &keys, name);
    }

    // create a list of files contained in the project, and create those files.
    // TODO should include templates/scripts/etc.
    let files = match project_files.files {
        // FIXME files need to have a newline insert in between them?
        Some(files) => render_files(files, &keys, name),
        None => VecBuilder::new(),
    };

    // create license if it was asked for
    if let Some(license) = license_contents {
        render_file(license, name, "LICENSE", &keys);
    }

    // render readme if requested
    if project.with_readme {
        render_file(includes::README, name, "README.md", &keys);
    }

    // Make a keys for inserting stuff into templates.
    keys = keys.insert("files", files);

    // render templates
    render_templates(&project.path, name, &keys, project_files.templates, false);

    // render scripts, i.e. files that should be executable.
    render_templates(&project.path, name, &keys, project_files.scripts, true);

    let version_control = project_config
        .and_then(|project_config| project_config.version_control)
        .or(config.version_control);

    // initialize version control
    if let Some(version_control) = version_control {
        match version_control {
            VersionControl::Git => git_init(name),
            VersionControl::Hg | VersionControl::Mercurial => hg_init(name),
            VersionControl::Pijul => pijul_init(name),
            VersionControl::Darcs => darcs_init(name),
            VersionControl::Unknown => warn!("Version control not yet supported, supported version control tools are git, darcs, pijul, and mercurial, ignoring...")
        }
    }

    Ok(())
}
