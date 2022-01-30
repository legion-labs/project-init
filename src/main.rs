//! Source file for the binary.

use std::fs::read_dir;

use args::Args;
use args::Subcommands;
use clap::StructOpt;
use git2::Repository;
use tempdir::TempDir;
use tracing::error;
use tracing_subscriber::FmtSubscriber;
use types::Config;
use types::Project;

use crate::constants::{
    GITHUB_URL, GLOBAL_CONFIG_FILENAME, GLOBAL_TEMPLATE_DIRECTORY, TEMPLATE_FILENAME,
};
use crate::util::init_helper;

mod args;
mod constants;
mod includes;
mod render;
mod repo;
mod types;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;

    let args = Args::parse();

    let home = dirs::home_dir().ok_or("Couldn't determine home directory")?;

    let config = Config::from_path(&home.join(GLOBAL_CONFIG_FILENAME));

    match args.subcommand {
        Subcommands::Git {
            repository,
            name,
            force,
        } => {
            let repository_url = match GITHUB_URL.join(&repository) {
                Ok(repository_url) => repository_url,
                Err(_) => {
                    error!("Failed to resolve the repository url");

                    std::process::exit(1);
                }
            };

            // create a temporary directory to hold the template
            let dir_name = repository.replace("/", "-");

            let tmp_directory = match TempDir::new(&dir_name) {
                Ok(tmp_directory) => tmp_directory,
                Err(_) => {
                    error!("Failed to create temporary directory");

                    std::process::exit(1);
                }
            };

            // clone into the temporary directory
            let directory = tmp_directory.path();

            if Repository::clone(repository_url.as_str(), directory).is_err() {
                error!("Failed to clone repository at {}", repository_url);

                std::process::exit(1);
            };

            // get the parsed TOML file from the repo.
            let project = Project::from_path(".", &directory);

            // initialize the project
            init_helper(&name, config, project, force)?;

            println!("Finished initializing project in {}", name);
        }

        Subcommands::New {
            directory,
            name,
            force,
        } => {
            let project = Project::from_path(&home, &directory);

            init_helper(&name, config, project, force)?;

            println!("Finished initializing project in {}", name);
        }

        Subcommands::List => {
            let local_templates_directory = home.join(GLOBAL_TEMPLATE_DIRECTORY);

            match read_dir(&local_templates_directory) {
                Ok(directories) => {
                    println!(
                        "Local templates located in {}",
                        local_templates_directory.to_string_lossy()
                    );

                    for directory in directories.flatten() {
                        let directory_path = directory.path();

                        if directory_path.is_dir() {
                            if let Some(directory_name) = directory_path.file_name() {
                                let template_toml_path = directory_path.join(TEMPLATE_FILENAME);

                                if template_toml_path.is_file() {
                                    println!("- pi new {}", directory_name.to_string_lossy());
                                }
                            }
                        }
                    }
                }

                Err(_error) => {
                    println!(
                        "No local templates found in {}",
                        local_templates_directory.to_string_lossy()
                    );
                }
            }

            match config.templates_repository {
                Some(templates_repository) => {
                    println!("Remote templates located in {}", templates_repository);

                    let entries = templates_repository.read().await;

                    if entries.is_empty() {
                        println!("No templates found in repository {}", templates_repository);
                    } else {
                        for entry in entries {
                            println!("- pi git {}", entry);
                        }
                    }
                }

                None => {
                    println!("No templates repository found in config")
                }
            }
        }
    }

    Ok(())
}
