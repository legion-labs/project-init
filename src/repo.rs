use std::process::Command;

use colored::*;

pub fn git_init(name: &str) {
    let mut cmd = "cd ".to_string();

    cmd.push_str(name);
    cmd.push_str("&&");
    cmd.push_str("git init && git add *");

    match Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .spawn()
    {
        Ok(c) => {
            c.wait_with_output().expect("failed to wait on child");
        }
        Err(_) => {
            eprintln!(
                "{}, git failed to initialize. Is git on your path?",
                "Error".red()
            );

            std::process::exit(0x0f01);
        }
    }
}

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
        Ok(c) => {
            c.wait_with_output().expect("failed to wait on child");
        }
        Err(_) => {
            eprintln!(
                "{}, Pijul failed to initialize. Is it on your path?",
                "Error".red()
            );

            std::process::exit(0x0f01);
        }
    }
}

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
        Ok(c) => {
            c.wait_with_output().expect("failed to wait on child");
        }
        Err(_) => {
            eprintln!(
                "{}, Darcs failed to initialize. Is hg on your path?",
                "Error".red()
            );

            std::process::exit(0x0f01);
        }
    }
}

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
        Ok(c) => {
            c.wait_with_output().expect("failed to wait on child");
        }
        Err(_) => {
            eprintln!(
                "{}, Mercurial failed to initialize. Is it on your path?",
                "Error".red()
            );

            std::process::exit(0x0f01);
        }
    }
}
