use lazy_static::lazy_static;
use url::Url;

lazy_static! {
    pub static ref GITHUB_URL: Url = "https://github.com".parse().unwrap();
}

pub const TEMPLATE_FILENAME: &str = "template.toml";

pub const GLOBAL_CONFIG_FILENAME: &str = ".pi.toml";

pub const GLOBAL_TEMPLATE_DIRECTORY: &str = ".pi_templates";
