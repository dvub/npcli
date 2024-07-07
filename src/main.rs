use anyhow::Result;
use clap::{Parser, Subcommand};
use cliclack::{input, intro, multiselect, note, select};
use nih_plug_xtask::bundle;
use nih_plug_xtask::{build, chdir_workspace_root};
use std::{env::current_dir, fs::File, io::Write, process::Command};
/// NIH-Plug CLI.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new NIH-Plug project through a step-by-step, interactive CLI.
    New,
    /// Compile an existing NIH-Plug project
    Bundle {
        /// Package(s) to compile.
        packages: Vec<String>,

        /// Any other arguments supported by cargo, such as profile arguments (`--release`), may be supplied here.
        /// To pass these arguments, you must first include `--`. E.g. `-- --release --profile ...`
        #[arg(raw = true)]
        other_args: Vec<String>,
    },

    BundleUniversal {
        packages: String,
    },
}

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
    // chdir_workspace_root()?;
    let cargo_metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path("./Cargo.toml")
        .exec()
        .unwrap(); // TODO
    let target_dir = cargo_metadata.target_directory.as_std_path();

    let args = Cli::parse();
    match args.command {
        Commands::New => create_project()?,

        Commands::Bundle {
            packages,
            other_args,
        } => {
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

fn create_project() -> Result<()> {
    let project_name: String = input("Project & Directory Name:")
        .placeholder("my_plugin")
        .required(true)
        .interact()
        .unwrap();

    // default values also are placeholders
    let default_vendor = "NIH-Plug";
    let default_name = "Gain";
    let default_url = "https://github.com/robbert-vdh/nih-plug";
    let default_email = "info@example.com";
    let default_vst_id = "Exactly16Chars!!";

    intro("create-nih-plug-project").unwrap();
    note("Note", "Press <Enter> for a field to use default value").unwrap();

    let plugin_name: String = input("Plugin Name: ()")
        .placeholder(default_name)
        .default_input(default_name)
        .interact()
        .unwrap();
    let vendor: String = input("Author/Vendor:")
        .placeholder(default_vendor)
        .default_input(default_vendor)
        .interact()
        .unwrap();
    let url: String = input("URL:")
        .placeholder(default_url)
        .default_input(default_url)
        .interact()
        .unwrap();
    let email: String = input("Email:")
        .placeholder(default_email)
        .default_input(default_email)
        .interact()
        .unwrap();
    let vst_id: String = input("VST ID:")
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
    let midi_config: String = select("MIDI Config:")
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
    let other_sub_categories = [
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
    let mut multi = multiselect("Other VST categories:");
    for cat in other_sub_categories {
        multi = multi.item(cat, cat, "");
    }
    multi.interact().unwrap();

    let current_dir = current_dir().unwrap();
    let project_path = current_dir.join(&project_name);

    // create a new project with cargo
    // TODO: make sure user has cargo installed
    let command = format!("cargo new --lib {}", project_name);

    // curse you windows!
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", &command])
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(&command)
            .output()
            .expect("failed to execute process")
    };

    /*
     * With Cargo.toml, we need to do some things
     * 1. add nih-plug using the git link
     * 2. indicate that this project is a cdylib
     */

    let mut file = File::options()
        .append(true)
        .open(project_path.join("Cargo.toml"))
        .unwrap();

    writeln!(file, "nih_plug = {{ git = \"https://github.com/robbert-vdh/nih-plug.git\", features = [\"assert_process_allocs\"] }}\n\n[lib]\ncrate-type = [\"cdylib\"]\n")
    .unwrap();

    let gitignore_path = project_path.join(".gitignore");
    let mut gitignore = File::create(gitignore_path).unwrap();
    gitignore.write_all(b"/target").unwrap();

    // TODO:
    // need readme

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
        sub_categories: "".to_string(),
    }
    .to_string();

    lib.write_all(output.as_bytes())
        .expect("Error writing file");

    Ok(())
}
