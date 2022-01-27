use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Subcommands {
    /// Fetch a template from github.
    #[clap(alias = "g")]
    Git {
        /// User and repository name where the template is located
        #[clap(value_name = "USER/REPO")]
        repo: String,
        /// Project name to be used for project directory.
        #[clap(value_name = "NAME")]
        name: String,
        /// Initialize project even if directory already exists.
        #[clap(long, short)]
        force: bool,
    },
    /// List available templates. User templates can be added by placing them in ~/.pi_templates
    #[clap(alias = "l")]
    List,
    /// Update pi (only works on UNIX).
    #[clap(alias = "u")]
    Update {
        /// Force installation even when binary already exists.
        #[clap(long, short)]
        force: bool,
    },
    /// Use a template from a folder.
    #[clap(alias = "i")]
    Init {
        /// Directory containing your template, either in the current directory or in $HOME/.pi_templates/
        #[clap(value_name = "TEMPLATE_DIR")]
        directory: String,
        /// Project name to be used for project directory.
        #[clap(value_name = "NAME")]
        name: String,
        /// Initialize project even if directory already exists.
        #[clap(long, short)]
        force: bool,
    },
    /// Use a built-in template.
    #[clap(alias = "n")]
    New {
        /// Template to used. Currently supported are Rust, Haskell, Idris, Elm, Python, Vimscript, Miso, and Julia.
        template: String,
        /// Project name to be used for project directory.
        #[clap(value_name = "NAME")]
        name: String,
        /// Initialize project even if directory already exists.
        #[clap(long, short)]
        force: bool,
    },
}
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, term_width = 80, after_help = "See 'man pi' for more information")]
pub struct Args {
    #[clap(subcommand)]
    pub subcommand: Subcommands,
}
