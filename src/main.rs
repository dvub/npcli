mod cli;
use anyhow::Result;
use clap::Parser;
use cli::*;

use cliclack::{input, intro, multiselect, select};
use nih_plug_xtask::build;
use nih_plug_xtask::bundle;
use std::env::set_current_dir;
use std::io::Read;
use std::path::Path;
use std::{env::current_dir, fs::File, io::Write, process::Command};
use toml::Value::String as TomlString;
use toml::Value::{Array, Table};
#[derive(boilerplate::Boilerplate)]
struct LibTxt {
    plugin_name: String,
    vendor: String,
    url: String,
    email: String,
    vst_id: String,
    midi_config: String,
    sub_categories: String,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Commands::New { skip_first_build } => create_project(skip_first_build)?,
        Commands::Bundle {
            packages,
            other_args,
        } => {
            // chdir_workspace_root()?;
            let cargo_metadata = cargo_metadata::MetadataCommand::new()
                .manifest_path("./Cargo.toml")
                .exec()
                .unwrap(); // TODO
            let target_dir = cargo_metadata.target_directory.as_std_path();

            build(&packages, &other_args)?;

            bundle(target_dir, &packages[0], &other_args, false)?;
            for package in packages.into_iter().skip(1) {
                bundle(target_dir, &package, &other_args, false)?;
            }
        }
        _ => todo!(),
    };

    Ok(())
}

fn create_project(skip_first_build: bool) -> Result<()> {
    // default values also are placeholders
    let default_vendor = "NIH-Plug";
    let default_name = "Gain";
    let default_url = "https://github.com/robbert-vdh/nih-plug";
    let default_email = "info@example.com";
    let default_vst_id = "Exactly16Chars!!";

    intro("create-nih-plug-project").unwrap();
    let project_name: String =
        input("What's your project named? (NOT the same as your plugin name)")
            .placeholder("gain")
            .required(true)
            .interact()
            .unwrap();

    let plugin_name: String = input("What's your plugin named?")
        .placeholder(default_name)
        .default_input(default_name)
        .interact()
        .unwrap();
    let vendor: String = input("Author?")
        .placeholder(default_vendor)
        .default_input(default_vendor)
        .interact()
        .unwrap();
    let url: String = input("URL?")
        .placeholder(default_url)
        .default_input(default_url)
        .interact()
        .unwrap();
    let email: String = input("Email?")
        .placeholder(default_email)
        .default_input(default_email)
        .interact()
        .unwrap();
    let vst_id: String = input("VST ID?")
        .placeholder(default_vst_id)
        .default_input(default_vst_id)
        .validate(|input: &String| {
            if input.len() != 16 {
                Err("VST3 ID must be exactly 16 characters.")
            } else {
                Ok(())
            }
        })
        .interact()
        .unwrap();
    let midi_config: String = select("MIDI Config?")
        .item("None", "None", "The plugin will not receive MIDI events.")
        .item("Basic", "Basic", "The plugin receives note on/off/choke events, pressure, and possibly standardized expression types.")
        .item(
            "MidiCC",
            "Full",
            "The plugin receives full MIDI CCs as well as pitch bend information.",
        )
        .interact()
        .unwrap()
        .to_owned();
    let main_vst_sub_category = select("Main VST category:")
        .item("Fx", "Fx", "")
        .item("Instrument", "Instrument", "")
        .item("Spatial", "Spatial", "")
        .interact()
        .unwrap();
    let other_sub_categories_list = [
        "Analyzer",
        "Delay",
        "Distortion",
        "Drum",
        "Dynamics",
        "Eq",
        "External",
        "Filter",
        "Generator",
        "Mastering",
        "Modulation",
        "Network",
        "Piano",
        "PitchShift",
        "Restoration",
        "Reverb",
        "Sampler",
        "Synth",
        "Tools",
        "UpDownmix",
    ];

    let mut multi_builder = multiselect("Other VST categories? (Optional)");
    for cat in other_sub_categories_list {
        multi_builder = multi_builder.item(cat, cat, "");
    }
    let mut other_sub_categories = multi_builder.required(false).interact().unwrap();

    let _wants_clap = select("Would you like to set up a CLAP export?")
        .item(true, "Yes!", "")
        .item(false, "No", "")
        .initial_value(false)
        .interact()
        .unwrap();

    // END OF USER INPUT

    other_sub_categories.insert(0, main_vst_sub_category);
    let concat_sub_categories: Vec<_> = other_sub_categories
        .iter()
        .map(|cat| format!("Vst3SubCategory::{}", cat))
        .collect();

    let vst_sub_categories = concat_sub_categories.join(", ");
    let current_dir = current_dir().unwrap();
    let project_path = current_dir.join(&project_name);

    cargo_new(&project_name);
    write_to_toml(&project_path)?;
    write_to_lib(
        &project_path,
        &LibTxt {
            plugin_name,
            vendor,
            url,
            email,
            vst_id,
            midi_config,
            sub_categories: vst_sub_categories,
        },
    );

    if skip_first_build {
        return Ok(());
    }

    let args = &["--release".to_owned()];
    set_current_dir(&project_path).unwrap();
    build(&[project_name.clone()], args)?;
    bundle(&project_path.join("target"), &project_name, args, false)?;

    Ok(())
}
// i may have overcomplicated this part by quite a lot,
// but eh
fn write_to_toml(project_path: &Path) -> Result<()> {
    // prereq: open file, read into a string, and parse the string with toml
    let mut file_read = File::options()
        .read(true)
        .open(project_path.join("Cargo.toml"))?;
    let mut str_contents = String::new();
    file_read.read_to_string(&mut str_contents)?;
    let mut value = str_contents.parse::<toml::Table>()?;

    // 1. add nih_plug as a dependency
    let dependencies = value
        .get_mut("dependencies")
        .unwrap()
        .as_table_mut()
        .unwrap();

    let mut nih_plug_table = toml::Table::new();
    nih_plug_table.insert(
        "git".to_owned(),
        TomlString("https://github.com/robbert-vdh/nih-plug.git".to_owned()),
    );
    dependencies.insert("nih_plug".to_owned(), Table(nih_plug_table));

    // declare that this is a cdylib
    let mut crate_type_table = toml::Table::new();
    crate_type_table.insert(
        "crate_type".to_owned(),
        Array(vec![TomlString("cdylib".to_owned())]),
    );
    value.insert("lib".to_owned(), Table(crate_type_table));

    // write it all back out
    let new_str = toml::to_string(&value).unwrap();
    // we must do this again to use truncate.
    // TODO: don't open file twice i guess
    let mut file_write = File::options()
        .truncate(true)
        .write(true)
        .open(project_path.join("Cargo.toml"))
        .unwrap();

    file_write.write_all(new_str.as_bytes()).unwrap();
    Ok(())
}

fn write_to_lib(project_path: &Path, data: &LibTxt) {
    // now we're going to generate our lib.rs file from our template and overwrite the existing lib.rs
    let lib_path = project_path.join("src").join("lib.rs");
    let mut lib_file = File::options().write(true).open(lib_path).unwrap();
    let output = data.to_string();
    lib_file
        .write_all(output.as_bytes())
        .expect("Error writing file");
}

fn cargo_new(project_name: &str) {
    // create a new project with cargo
    // TODO: make sure user has cargo installed
    let command = format!("cargo new --lib {} --vcs git", project_name);
    exec_command(&command);
}

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
