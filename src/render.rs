//! Module containing functions for rendering templates

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use os_str_bytes::OsStrBytes;
use rustache::*;
use tracing::error;

/// Trait allowing us to create dirs/templates/files.
trait Create {
    fn create_dirs<P: AsRef<Path>>(&self, name: P);
}

/// Create directories given a `Vec<AsRes<Path>>` of directory names
impl<T: AsRef<Path>> Create for Vec<T> {
    fn create_dirs<P: AsRef<Path>>(&self, name: P) {
        self.iter().for_each(|dir| {
            let subdir = name.as_ref().join(dir);

            let _ = fs::create_dir(subdir);
        });
    }
}

/// Render a list of directories, substituting in templates
pub fn render_dirs<D: AsRef<Path>, N: AsRef<Path>>(
    directories: Vec<D>,
    hash: &HashBuilder,
    name: N,
) {
    // substitute into directory names using templates
    let directories: Vec<String> = directories
        .into_iter()
        .map(|file| {
            let mut output = Cursor::new(Vec::new());

            hash.render(&file.as_ref().to_string_lossy(), &mut output)
                .unwrap();

            String::from_utf8(output.into_inner()).unwrap()
        })
        .collect();

    directories.create_dirs(name);
}

/// Create all the files, and return a list of files that have been created
/// suitable for insertion
/// into a `HashBuilder`
pub fn render_files<'a, D: AsRef<Path>, N: AsRef<Path>>(
    files: Vec<D>,
    hash: &HashBuilder,
    name: N,
) -> VecBuilder<'a> {
    // render filenames
    let substitutions = files
        .into_iter()
        .map(|file| {
            let mut output = Cursor::new(Vec::new());

            hash.render(&file.as_ref().to_string_lossy(), &mut output)
                .unwrap();

            Path::from_raw_bytes(output.into_inner())
                .unwrap()
                .as_ref()
                .to_path_buf()
        })
        .collect::<Vec<PathBuf>>();

    // create files
    substitutions.iter().for_each(|path| {
        File::create(name.as_ref().join(path)).unwrap();
    });

    // collect filenames
    let data: Vec<Data> = substitutions
        .into_iter()
        .map(|substitution| Data::from(substitution.to_string_lossy().into_owned()))
        .collect();

    // return a `VecBuilder` object.
    VecBuilder { data }
}

/// render a `<Vec<String>>` of templates, doing nothing if it's empty.
#[cfg(target_os = "windows")]
pub fn render_templates<P: AsRef<Path>, T: AsRef<Path>, N: AsRef<Path>>(
    project_path: P,
    name: N,
    hash: &HashBuilder,
    templates: Option<Vec<T>>,
    executable: bool,
) {
    if let Some(original_templates) = templates {
        // create Vec<T> of paths to templates
        let templates = original_templates
            .iter()
            .map(|file| {
                let mut path = project_path.as_ref().join(file);

                if executable {
                    path = path.join(".bat");
                }

                path
            })
            .collect::<Vec<PathBuf>>();

        // read all the template files
        let template_files = templates
            .iter()
            .map(|path| {
                let mut template_file = match File::open(&path) {
                    Ok(template_file) => template_file,
                    Err(_) => {
                        error!("Failed to open file: {:?}", path);

                        std::process::exit(0x0f00);
                    }
                };

                let mut template = String::new();

                template_file
                    .read_to_string(&mut template)
                    // ok to panic because we already errored.
                    .expect("File read failed");

                template
            })
            .collect::<Vec<String>>();

        // create Vec<T> of paths to rendered templates
        let templates_new = original_templates
            .iter()
            .map(|file| name.as_ref().join(file))
            .collect::<Vec<PathBuf>>();

        // subtitute into template names
        let templates_named = templates_new
            .iter()
            .map(|name| {
                let mut output = Cursor::new(Vec::new());

                hash.render(&name.to_string_lossy(), &mut output).unwrap();

                Path::from_raw_bytes(output.into_inner())
                    .unwrap()
                    .as_ref()
                    .to_path_buf()
            })
            .collect::<Vec<PathBuf>>();

        // render all the template files
        let substitutions = template_files
            .iter()
            .map(|file| {
                let mut output = Cursor::new(Vec::new());

                hash.render(file, &mut output).unwrap();

                output.into_inner()
                // Path::from_raw_bytes(output.into_inner()).unwrap().as_ref()
            })
            .collect::<Vec<Vec<u8>>>();

        // write the rendered templates
        let files_to_write = templates_named.iter().zip(substitutions.iter());

        files_to_write
            .into_iter()
            .for_each(|(path, contents)| match File::create(&path) {
                Ok(mut file) => {
                    let _ = file.write(contents);
                }
                Err(_error) => {
                    error!("Failed to create file: {:?}, check that the directory is included in your template.toml", path);

                    std::process::exit(0x0f01);
                }
            });
    }
}

/// render a `<Vec<String>>` of templates, doing nothing if it's empty.
#[cfg(not(target_os = "windows"))]
pub fn render_templates<P: AsRef<Path>, T: AsRef<Path>, N: AsRef<Path>>(
    project_path: P,
    name: N,
    hash: &HashBuilder,
    templates: Option<Vec<T>>,
    executable: bool,
) {
    if let Some(original_templates) = templates {
        // create Vec<T> of paths to templates
        let templates = original_templates
            .iter()
            .map(|file| project_path.as_ref().join(file))
            .collect::<Vec<PathBuf>>();

        // read all the template files
        let template_files = templates
            .iter()
            .map(|path| {
                let mut template_file = match File::open(&path) {
                    Ok(template_file) => template_file,
                    Err(_) => {
                        error!("Failed to open file: {:?}", path);

                        std::process::exit(0x0f00);
                    }
                };

                let mut template = String::new();

                template_file
                    .read_to_string(&mut template)
                    // ok to panic because we already errored.
                    .expect("File read failed");

                template
            })
            .collect::<Vec<String>>();

        // create Vec<T> of paths to rendered templates
        let templates_new = original_templates
            .iter()
            .map(|file| name.as_ref().join(file))
            .collect::<Vec<PathBuf>>();

        // subtitute into template names
        let templates_named = templates_new
            .iter()
            .map(|name| {
                let mut output = Cursor::new(Vec::new());

                hash.render(&name.to_string_lossy(), &mut output).unwrap();

                Path::from_raw_bytes(output.into_inner())
                    .unwrap()
                    .as_ref()
                    .to_path_buf()
            })
            .collect::<Vec<PathBuf>>();

        // render all the template files
        let substitutions = template_files
            .iter()
            .map(|file| {
                let mut output = Cursor::new(Vec::new());

                hash.render(file, &mut output).unwrap();

                output.into_inner()
                // Path::from_raw_bytes(output.into_inner()).unwrap().as_ref()
            })
            .collect::<Vec<Vec<u8>>>();

        // write the rendered templates
        let files_to_write = templates_named.iter().zip(substitutions.iter());

        files_to_write
            .into_iter()
            .for_each(|(path, contents)| match File::create(&path) {
                Ok(mut file) => {
                    let _ = file.write(contents);

                    if executable {
                        let mut permissions = fs::metadata(path)
                            .expect("failed to read file metadata")
                            .permissions();

                        permissions.set_mode(0o755);

                        let _ = fs::set_permissions(path, permissions);
                    };
                }
                Err(_error) => {
                    error!("Failed to create file: {:?}, check that the directory is included in your template.toml", path);

                    std::process::exit(0x0f01);
                }
            });
    }
}

/// Render a static string and write it to file
pub fn render_file<N: AsRef<Path>>(
    static_template: &str,
    name: N,
    filename: &str,
    hash: &HashBuilder,
) {
    // render the template
    let mut output = Cursor::new(Vec::new());

    hash.render(static_template, &mut output).unwrap();

    let contents = String::from_utf8(output.into_inner()).unwrap();

    // write the file
    let path = name.as_ref().join(filename);

    // write the rendered template
    match File::create(&path) {
        Ok(mut file) => {
            let _ = file.write(contents.as_bytes());
        }
        Err(_) => {
            error!(
                "Failed to create file: {:?}. Check that the directory is included in your template.toml",
                path
            );

            std::process::exit(0x0f01);
        }
    }
}
