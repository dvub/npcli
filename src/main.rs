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

// TODO:
// - add more comments
// - add documentation
// - finish bundle/bundle-universal
// - finish new - almost done, just have to implement standalone export

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

#[derive(boilerplate::Boilerplate)]
struct ClapTxt {
    plugin_name: String,
    clap_id: String,
    clap_description: String,
    clap_features: String,
}

#[derive(Clone, PartialEq, Eq)]
enum ExportTypes {
    Clap,
    Standalone,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Commands::New {
            skip_build,
            defaults,
            name,
        } => create_project(name, defaults, skip_build)?,
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

fn create_project(name: Option<String>, defaults: bool, skip_first_build: bool) -> Result<()> {
    // TODO: at this top level, instead of using ? operator, actually write user-facing errors!!

    // default values also are placeholders
    let default_vendor = "NIH-Plug";
    let default_name = "Gain";
    let default_url = "https://github.com/robbert-vdh/nih-plug";
    let default_email = "info@example.com";
    let default_vst_id = "Exactly16Chars!!";
    let default_midi_config = "None";
    let default_sub_category = "Vst3SubCategory::Fx";
    // if the user supplies `--defaults`, we will use these
    let mut clap_data = None;
    let mut lib = LibTxt {
        plugin_name: default_name.to_string(),
        vendor: default_vendor.to_owned(),
        url: default_url.to_owned(),
        email: default_email.to_owned(),
        vst_id: default_vst_id.to_owned(),
        midi_config: default_midi_config.to_owned(),
        sub_categories: default_sub_category.to_owned(),
    };

    let project_name: String = name.unwrap_or(
        input("What's your project named? (NOT the same as your plugin name)")
            .placeholder("gain")
            .required(true)
            .interact()?,
    );

    if !defaults {
        intro("create-nih-plug-project").unwrap();

        let plugin_name: String = input("What's your plugin named?")
            .placeholder(default_name)
            .default_input(default_name)
            .interact()?;
        let vendor: String = input("Author?")
            .placeholder(default_vendor)
            .default_input(default_vendor)
            .interact()?;
        let url: String = input("URL?")
            .placeholder(default_url)
            .default_input(default_url)
            .interact()?;
        let email: String = input("Email?")
            .placeholder(default_email)
            .default_input(default_email)
            .interact()?;
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
            .interact()?;
        let midi_config: String = select("MIDI Config?")
        .item("None", "None", "The plugin will not receive MIDI events.")
        .item("Basic", "Basic", "The plugin receives note on/off/choke events, pressure, and possibly standardized expression types.")
        .item(
            "MidiCC",
            "Full",
            "The plugin receives full MIDI CCs as well as pitch bend information.",
        )
        .initial_value("None")
        .interact()?
        .to_owned();

        let vst_sub_categories = build_category_list(
            "Main VST Subcategory?",
            vec!["Fx", "Instrument", "Spatial"],
            "Other VST Subcategories?",
            vec![
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
            ],
            "Vst3SubCategory",
        )?;

        let other_export_types = multiselect("Other export types?")
            .item(
                ExportTypes::Clap,
                "CLAP",
                "See https://cleveraudio.org/ for more info",
            )
            .item(
                ExportTypes::Standalone,
                "Standalone",
                "Creates a standalone application that can run outside of a DAW/VST host",
            )
            .required(false)
            .interact()?;

        // we need a series of prompts to handle CLAP export configuration.

        if other_export_types.contains(&ExportTypes::Clap) {
            let default_clap_id = "com.moist-plugins-gmbh.gain";
            let default_clap_description = "A smoothed gain parameter example plugin";
            // clap id
            let clap_id: String = input("CLAP ID?")
                .placeholder(default_clap_id)
                .default_input(default_clap_id)
                .interact()?;

            // clap description
            let clap_description: String = input("CLAP ID?")
                .placeholder(default_clap_description)
                .default_input(default_clap_description)
                .interact()?;

            // clap features
            let clap_features = build_category_list(
                "Main CLAP Feature?",
                vec!["Instrument", "AudioEffect", "NoteDetector", "NoteEffect"],
                "Other CLAP Features?",
                vec![
                    "Analyzer",
                    "Synthesizer",
                    "Sampler",
                    "Drum",
                    "DrumMachine",
                    "Filter",
                    "Phaser",
                    "Equalizer",
                    "Deesser",
                    "PhaseVocoder",
                    "Granular",
                    "FrequencyShifter",
                    "PitchShifter",
                    "Distortion",
                    "TransientShaper",
                    "Compressor",
                    "Expander",
                    "Gate",
                    "Limiter",
                    "Flanger",
                    "Chorus",
                    "Delay",
                    "Reverb",
                    "Tremolo",
                    "Glitch",
                    "Utility",
                    "PitchCorrection",
                    "Restoration",
                    "MultiEffects",
                    "Mixing",
                    "Mastering",
                    "Mono",
                    "Stereo",
                    "Surround",
                    "Ambisonic",
                ],
                "ClapFeature",
            )?;

            clap_data = Some(ClapTxt {
                plugin_name: plugin_name.clone(),
                clap_id,
                clap_description,
                clap_features: clap_features.to_owned(),
            });
        }
        if other_export_types.contains(&ExportTypes::Standalone) {}

        lib = LibTxt {
            plugin_name,
            vendor,
            url,
            email,
            vst_id,
            midi_config,
            sub_categories: vst_sub_categories,
        };
    }
    // END OF USER INPUT

    // now, create/modify files
    let current_dir = current_dir().unwrap();
    let project_path = current_dir.join(&project_name);
    cargo_new(&project_name);
    write_to_toml(&project_path)?;
    write_to_lib(&project_path, &lib, clap_data)?;

    if skip_first_build {
        return Ok(());
    }

    // finally, build the plugin
    let args = &["--release".to_owned()];
    set_current_dir(&project_path)?;
    build(&[project_name.clone()], args)?;
    bundle(&project_path.join("target"), &project_name, args, false)?;

    Ok(())
}
// why did i document this so much??

/// Creates a `select` and `multi-select` for a main category and optional categories.
/// Returns a list of categories as a string, used for VST3 Subcategories and CLAP features.
/// ## Parameters
/// - `main_category_prompt`: A prompt that the user will see for the main, required category.
/// - `main_category_list`: This vec will be used as the options to build a `select` for the main category.
/// - `multi_select_prompt`: A prompt that the user will see for the remaining optional categories.
/// - `other_categories_list`: This vec will be used as the options to build a `multi-select` for the optional categories.
/// - `enum_prefix`: When the final list is stringified, you may prepend a string version of an enum to each category.
fn build_category_list(
    main_category_prompt: &str,
    main_category_list: Vec<&str>,
    multi_select_prompt: &str,
    other_categories_list: Vec<&str>,
    enum_prefix: &str,
) -> Result<String> {
    // main, required category
    let mut main_category_select = select(main_category_prompt);
    for item in main_category_list {
        main_category_select = main_category_select.item(item, item, "");
    }
    let main_category = main_category_select.interact()?;

    // other, optional categories
    let mut multi_builder = multiselect(multi_select_prompt);
    for cat in other_categories_list {
        multi_builder = multi_builder.item(cat, cat, "");
    }
    let mut other_categories = multi_builder.required(false).interact()?;
    // add main category to beginning of list
    other_categories.insert(0, main_category);
    let concat_items: Vec<_> = other_categories
        .iter()
        // prepend an enum
        .map(|feature| format!("{}::{}", enum_prefix, feature))
        .collect();
    // done!!
    Ok(concat_items.join(", "))
}

// i may have overcomplicated this part by quite a lot,
// but eh
// an easier thing to do would have been to use a templated Cargo.toml file or something.

/// Opens an existing Cargo.toml file, adds the `nih_plug` crate (with the github link),
/// and adds the `cdylib` crate type.
fn write_to_toml(project_path: &Path) -> Result<()> {
    // TODO:
    // figure out how to deal with all of these unwrap() calls
    //
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

    // 2. declare that this is a cdylib
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

/// Takes user input and generates a lib.rs file.
/// The user input includes general plugin information, as well as optional CLAP info.
fn write_to_lib(project_path: &Path, data: &LibTxt, clap_data: Option<ClapTxt>) -> Result<()> {
    // now we're going to generate our lib.rs file from our template and overwrite the existing lib.rs
    let lib_path = project_path.join("src").join("lib.rs");
    let mut lib_file = File::options().write(true).open(lib_path)?;
    let mut output = data.to_string();

    // if the user configured CLAP, add it to the file.
    if let Some(data) = clap_data {
        let clap_output = data.to_string();
        output.push_str(&clap_output);
    }

    lib_file.write_all(output.as_bytes())?;

    Ok(())
}

/// Executes the `cargo new` command, creating a new project.
/// **NOTE**: this function creates the new project *with a git repo* (via `--vcs git`)
fn cargo_new(project_name: &str) {
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
