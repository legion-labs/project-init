use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Subcommands {
    /// Fetch a template from github.
    #[clap(alias = "g")]
    Git {
        /// User and repository name where the template is located
        #[clap(value_name = "USER/REPO")]
        repository: String,
        /// Project name to be used for project directory.
        #[clap(value_name = "NAME")]
        name: String,
        /// Initialize project even if directory already exists.
        #[clap(long, short)]
        force: bool,
    },
    /// Use a template from a folder.
    #[clap(alias = "n")]
    New {
        /// Directory containing your template, either in the current directory or in $HOME/.pi_templates/
        #[clap(value_name = "TEMPLATE_DIR")]
        directory: PathBuf,
        // TODO: We should probably disambiguate between the name and the output dir at one point
        /// Project name to be used for project directory.
        #[clap(value_name = "NAME")]
        name: String,
        /// Initialize project even if directory already exists.
        #[clap(long, short)]
        force: bool,
    },
    /// List all the available templates remotely and in the $HOME/.pi_templates/ directory
    #[clap(alias = "ls")]
    List,
}
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, term_width = 80, after_help = "See 'man pi' for more information")]
pub struct Args {
    #[clap(subcommand)]
    pub subcommand: Subcommands,
}
