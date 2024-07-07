mod cli;
use anyhow::Result;
use clap::Parser;
use cli::*;
use cliclack::progress_bar;
use cliclack::{input, intro, multiselect, select};
use nih_plug_xtask::build;
use nih_plug_xtask::bundle;
use std::{env::current_dir, fs::File, io::Write, process::Command};

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
        Commands::New {
            first_build: should_build,
            other_args,
        } => create_project(should_build, other_args)?,
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

fn create_project(should_build: bool, _other_args: Vec<String>) -> Result<()> {
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

    other_sub_categories.insert(0, main_vst_sub_category);
    let concat_sub_categories: Vec<_> = other_sub_categories
        .iter()
        .map(|cat| format!("Vst3SubCategory::{}", cat))
        .collect();

    let vst_sub_categories = concat_sub_categories.join(", ");
    // END OF USER INPUT
    let current_dir = current_dir().unwrap();
    let project_path = current_dir.join(&project_name);

    let progress = progress_bar(100);
    progress.start("Creating a new plugin...");

    progress.stop("Done!");

    // create a new project with cargo
    // TODO: make sure user has cargo installed
    let command = format!("cargo new --lib {} --vcs git", project_name);
    exec_command(&command);

    /*
     * With Cargo.toml, we need to do some things
     * 1. add nih-plug using the git link
     * 2. indicate that this project is a cdylib
     */

    // TODO:
    // is there a better way to do this??
    // possibly with toml crate
    let mut file = File::options()
        .append(true)
        .open(project_path.join("Cargo.toml"))
        .unwrap();

    writeln!(file, "nih_plug = {{ git = \"https://github.com/robbert-vdh/nih-plug.git\", features = [\"assert_process_allocs\"] }}\n\n[lib]\ncrate-type = [\"cdylib\"]\n")
    .unwrap();
    // TODO:
    // need readme?

    // now we're going to generate our lib.rs file from our template and overwrite the existing lib.rs
    let lib_path = project_path.join("src").join("lib.rs");
    let mut lib = File::options().write(true).open(lib_path).unwrap();
    let output = LibTxt {
        plugin_name,
        vendor,
        url,
        email,
        vst_id,
        midi_config,
        sub_categories: vst_sub_categories,
    }
    .to_string();

    lib.write_all(output.as_bytes())
        .expect("Error writing file");

    if should_build {
        println!("COMPILING...");
        todo!();
    }

    Ok(())
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
