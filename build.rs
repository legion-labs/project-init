use cli_setup::*;

const MAN_PI: &str = "man/pi.1";

const MAN_PI_CONTENT: &str = include_str!("man/pi.1");

fn main() {
    println!("cargo:rerun-if-changed={}", MAN_PI);

    setup_manpages(MAN_PI_CONTENT, "pi");
}
