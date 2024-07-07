use anyhow::Result;
use clap::{Parser, Subcommand};
use cliclack::{input, intro, note};
use nih_plug_xtask::bundle;
use nih_plug_xtask::{build, chdir_workspace_root};
use std::{env::current_dir, fs::File, io::Write, process::Command};
/// NIH-Plug CLI.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(trailing_var_arg = true)]
    args: Vec<String>,
}
#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new NIH-Plug project (interactive)
    New,
    /// Compile an existing NIH-Plug project
    Bundle {
        packages: Vec<String>,
    },

    BundleUniversal {
        packages: Vec<String>,
    },
}

#[derive(boilerplate::Boilerplate)]
struct LibTxt {
    plugin_name: String,
    vendor: String,
    url: String,
    email: String,
    vst_id: String,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    println!("{:?}", args.args);

    chdir_workspace_root()?;
    let cargo_metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path("./Cargo.toml")
        .exec()
        .unwrap(); // TODO: dont unwrap
    let target_dir = cargo_metadata.target_directory.as_std_path();

    match args.command {
        Commands::Bundle { packages } => {
            build(&packages, &[String::new()])?;
            bundle(target_dir, &packages[0], &[String::new()], false)?;
            for package in packages.into_iter().skip(1) {
                bundle(target_dir, &package, &[String::new()], false)?;
            }
            Ok(())
        }
        _ => todo!(),
    }
}

fn compile(name: String) {
    let output = Command::new("cmd")
        .args([
            "/C",
            &format!(
                "cargo run --package xtask --release -- bundle {} --release",
                name
            ),
        ])
        .output()
        .expect("failed to execute process");

    println!("{:?}", String::from_utf8(output.stdout).unwrap());
    println!("{:?}", String::from_utf8(output.stderr).unwrap());
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
    let vst_id: String = input("VST ID")
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

    let current_dir = current_dir().unwrap();
    let project_path = current_dir.join(&project_name);

    // create a new project with cargo
    // TODO: make sure user has cargo installed

    Command::new("cmd")
        .args(["/C", &format!("cargo new --lib {}", project_name)])
        .current_dir(&current_dir)
        .output()
        .expect("failed to execute process");

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
    }
    .to_string();
    lib.write_all(output.as_bytes())
        .expect("Error writing file");

    Ok(())
}
