//! Source file for the binary.

use args::Args;
use args::Subcommands;
use clap::StructOpt;
use git2::Repository;
use lazy_static::lazy_static;
use tempdir::TempDir;
use tracing::error;
use tracing_subscriber::FmtSubscriber;
use types::Config;
use types::Project;
use url::Url;

use crate::util::init_helper;

mod args;
mod includes;
mod render;
mod repo;
mod types;
mod util;

lazy_static! {
    static ref GITHUB_URL: Url = "https://github.com".parse().unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;

    let args = Args::parse();

    let home = dirs::home_dir().ok_or("Couldn't determine home directory")?;

    let config = Config::from_path(&home.join(".pi.toml"));

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
    }

    Ok(())
}
