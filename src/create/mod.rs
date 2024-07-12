mod boilerplate;
mod config;
mod gen;

use anyhow::Result;
use boilerplate::{LibConfig, StandaloneConfig, Vst3Config};
use cliclack::log::{error, info};
use cliclack::{confirm, input};
use colored::Colorize;
use config::{collect_export_types, configure_lib};
use config::{configure_clap_export, configure_vst_export};
use gen::{cargo_new, write_to_lib, write_to_main, write_to_toml};
use nih_plug_xtask::{build, bundle};
use std::env::current_dir;
use std::env::set_current_dir;
use std::fs::remove_dir_all;
// is a whole enum for this really needed?
#[derive(Clone, PartialEq, Eq)]
enum ExportType {
    Vst3,
    Clap,
    Standalone,
}
/// Creates a new nih-plug project based on the following parameters:
/// ## Parameters
/// - `name`: The name of the *project* - that being the directory/crate to be created.
/// - `defaults`: Setting this to true will skip any user input and just create/compile the plugin.
/// - `skip_first_build`: Setting this option will skip the first compilation. May be helpful to save some time.
pub fn create_project(name: Option<String>, defaults: bool, skip_first_build: bool) -> Result<()> {
    // TODO: at this top level, instead of using ? operator, actually write user-facing errors!!

    // if the user supplies `--defaults`, we will use these
    let mut lib_config = LibConfig::default();
    // vst_data has to be initialized so that we have at least one export by default
    let mut vst_config = Some(Vst3Config::default());
    // clap/standalone exports are not defaults, so initialize them to None
    let mut clap_config = None;
    let mut standalone_config = None;

    let project_name: String = if let Some(name) = name {
        name
    } else {
        let name_prompt = format!(
            "What's your {} named? ({} the same as your {})",
            "project".bold(),
            "not".bold(),
            "plugin's name".italic()
        );
        input(name_prompt)
            .required(true)
            .placeholder("gain")
            .interact()?
    };

    let current_dir = current_dir().unwrap();
    let path = current_dir.join(&project_name);

    if path.exists() {
        let delete_prompt = format!(
            "The directory \"{}\" already exists. Would you like to overwrite it? {}",
            project_name,
            "This will delete all existing contents!".red().bold()
        );
        let delete_dir = confirm(delete_prompt).initial_value(false).interact()?;

        let absolutely_sure = confirm("Are you absolutely sure?".italic())
            .initial_value(false)
            .interact()?;

        if delete_dir && absolutely_sure {
            info("Removing contents...")?;
            remove_dir_all(&path)?;
        } else {
            info("Exiting... If you'd like to continue, please choose a different project/directory name.")?;
            return Ok(());
        }
    }

    // TODO:
    // there is probably a way to refactor this to save on this indent,
    // i just didnt figure it out :(

    if !defaults {
        lib_config = configure_lib()?;
        let plugin_name = lib_config.plugin_name.clone();
        // beyond the basic info, we need to know which exports to set up
        let export_types = collect_export_types();
        // update vst config
        vst_config = if export_types.contains(&ExportType::Vst3) {
            Some(configure_vst_export(&plugin_name)?)
        } else {
            // since VST is the default type,
            // if the user UN-selects VST, we need to consider that
            None
        };
        // handle CLAP configuration/code generation
        if export_types.contains(&ExportType::Clap) {
            clap_config = Some(configure_clap_export(&plugin_name)?);
        }

        // finally, standalone setup
        if export_types.contains(&ExportType::Standalone) {
            standalone_config = Some(StandaloneConfig {
                // TODO: don't clone here, this looks stupid
                plugin_name,
                project_name: project_name.to_string(),
            });
        }
    }

    // now, create/modify files
    cargo_new(&project_name);
    println!("Created a new project...");

    write_to_toml(standalone_config.is_some(), &path)?;
    println!("Updated Cargo.toml...");

    write_to_lib(&path, &lib_config, clap_config, vst_config)?;
    println!("Updated lib.rs...");

    write_to_main(&path, standalone_config)?;
    //.unwrap_or(error("There was an error writing to main.rs")?);
    println!("Created main.rs...");

    if skip_first_build {
        return Ok(());
    }

    println!("Beginning build...");
    // finally, build the plugin
    let args = &["--release".to_owned()];
    set_current_dir(&path)?;
    build(&[project_name.clone()], args)?;
    bundle(&path.join("target"), &project_name, args, false)?;

    Ok(())
}
