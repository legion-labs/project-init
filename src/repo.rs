use std::process::Command;

use tracing::error;

pub fn git_init(name: &str) {
    if git2::Repository::init(name).is_err() {
        error!("Git failed to initialize, is it in your path?");

        std::process::exit(0x0f01);
    }
}

// FIXME: This function doesn't work on Windows
pub fn pijul_init(name: &str) {
    let mut cmd = "cd ".to_string();

    cmd.push_str(name);
    cmd.push_str("&&");
    cmd.push_str("pijul init && pijul add **");

    match Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .spawn()
    {
        Ok(child) => {
            child.wait_with_output().expect("failed to wait on child");
        }
        Err(_error) => {
            error!("Pijul failed to initialize, is it in your path?");

            std::process::exit(0x0f01);
        }
    }
}

// FIXME: This function doesn't work on Windows
pub fn darcs_init(name: &str) {
    let mut cmd = "cd ".to_string();

    cmd.push_str(name);
    cmd.push_str("&&");
    cmd.push_str("darcs init && darcs add **");

    match Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .spawn()
    {
        Ok(child) => {
            child.wait_with_output().expect("failed to wait on child");
        }
        Err(_error) => {
            error!("Darcs failed to initialize, is it in your path?");

            std::process::exit(0x0f01);
        }
    }
}

// FIXME: This function doesn't work on Windows
pub fn hg_init(name: &str) {
    let mut cmd = "cd ".to_string();

    cmd.push_str(name);
    cmd.push_str("&&");
    cmd.push_str("hg init && hg add *");

    match Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .spawn()
    {
        Ok(child) => {
            child.wait_with_output().expect("failed to wait on child");
        }
        Err(_error) => {
            error!("Mercurial failed to initialize, is it in your path?");

            std::process::exit(0x0f01);
        }
    }
}
