use super::boilerplate::{ClapConfig, LibConfig, StandaloneConfig, Vst3Config};
use anyhow::Result;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::{fs::File, io::Write};
use toml::Table;
use toml::Value::String as TomlString;
use toml::Value::{Array, Table as VTable};

// i may have overcomplicated this part by quite a lot,
// but eh
// an easier thing to do would have been to use a templated Cargo.toml file or something.

/// Opens an existing Cargo.toml file, adds the `nih_plug` crate (with the github link),
/// and adds the `cdylib` crate type.
pub fn write_to_toml<P: AsRef<Path>>(standalone: bool, project_path: P) -> Result<()> {
    // TODO:
    // figure out how to deal with all of these unwrap() calls
    //
    // prereq: open file, read into a string, and parse the string with toml
    let mut file_read = File::options()
        .read(true)
        .open(project_path.as_ref().join("Cargo.toml"))?;
    let mut str_contents = String::new();
    file_read.read_to_string(&mut str_contents)?;
    let mut value = str_contents.parse::<toml::Table>()?;

    // 1. add nih_plug as a dependency
    let dependencies = value
        .get_mut("dependencies")
        .unwrap()
        .as_table_mut()
        .unwrap();

    add_nih_plug(dependencies, standalone);

    let mut crate_type = vec![TomlString("cdylib".to_owned())];
    if standalone {
        crate_type.push(TomlString("lib".to_owned()));
    }

    // 2. declare that this is a cdylib
    let mut crate_type_table = toml::Table::new();
    crate_type_table.insert("crate_type".to_owned(), Array(crate_type));
    value.insert("lib".to_owned(), VTable(crate_type_table));

    // write it all back out
    let new_str = toml::to_string(&value).unwrap();
    // we must do this again to use truncate.
    // TODO: don't open file twice i guess
    let mut file_write = File::options()
        .truncate(true)
        .write(true)
        .open(project_path.as_ref().join("Cargo.toml"))
        .unwrap();

    file_write.write_all(new_str.as_bytes()).unwrap();
    Ok(())
}

fn add_nih_plug(dependencies: &mut Table, standalone: bool) {
    let mut nih_plug_table = toml::Table::new();
    nih_plug_table.insert(
        "git".to_owned(),
        TomlString("https://github.com/robbert-vdh/nih-plug.git".to_owned()),
    );

    // program will panic if allocation occurs on the process thread
    // we want this feature no matter what
    let mut features_vec = vec![TomlString("assert_process_allocs".to_owned())];

    // unlike assert_process_allocs above, we only include this feature if the user wants
    if standalone {
        features_vec.push(TomlString("standalone".to_owned()));
    }
    nih_plug_table.insert("features".to_owned(), Array(features_vec));
    dependencies.insert("nih_plug".to_owned(), VTable(nih_plug_table));
}

pub fn write_to_main<P: AsRef<Path>>(
    project_path: P,
    standalone_config: Option<StandaloneConfig>,
) -> Result<()> {
    // 99.9% sure that create() is ok since the file probably won't already exist
    if let Some(main) = standalone_config {
        let mut main_file = File::create(project_path.as_ref().join("src").join("main.rs"))?;
        main_file.write_all(main.to_string().as_bytes())?;
    }
    Ok(())
}

/// Takes user input and generates a lib.rs file.
/// The user input includes general plugin information, as well as optional CLAP info.
pub fn write_to_lib<P: AsRef<Path>>(
    project_path: P,
    lib_config: &LibConfig,
    clap_config: Option<ClapConfig>,
    vst_config: Option<Vst3Config>,
) -> Result<()> {
    // now we're going to generate our lib.rs file from our template and overwrite the existing lib.rs
    let lib_path = project_path.as_ref().join("src").join("lib.rs");
    let mut lib_file = File::options().write(true).open(lib_path)?;
    let mut output = lib_config.to_string();

    // if the user configured CLAP, add it to the file.
    if let Some(data) = clap_config {
        output.push_str(&data.to_string());
    }
    // if the user configured CLAP, add it to the file.
    if let Some(data) = vst_config {
        output.push_str(&data.to_string());
    }

    lib_file.write_all(output.as_bytes())?;

    Ok(())
}

/// Executes the `cargo new` command, creating a new project.
/// **NOTE**: this function creates the new project *with a git repo* (via `--vcs git`)
pub fn cargo_new(project_name: &str) {
    // creates a new project with cargo
    // TODO: make sure user has cargo installed
    let command = format!("cargo new --lib {} --vcs git", project_name);
    exec_command(&command);
}

// is this over-engineering?

/// Executes the given command based on the current platform.
fn exec_command(command: &str) {
    let (proc, arg) = if cfg!(target_os = "windows") {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };
    Command::new(proc)
        .args([arg, command])
        .output()
        .expect("Error running command");
}
