mod boilerplate;
mod export;
mod gen;

use anyhow::Result;
use boilerplate::{LibConfig, StandaloneConfig, Vst3Config};
use cliclack::{input, intro, multiselect, select};
use export::{configure_clap_export, configure_vst_export};
use gen::{cargo_new, write_to_lib, write_to_main, write_to_toml};
use nih_plug_xtask::{build, bundle};
use std::env::current_dir;
use std::env::set_current_dir;

const DEFAULT_VENDOR: &str = "NIH-Plug";
const DEFAULT_NAME: &str = "Gain";
const DEFAULT_URL: &str = "https://github.com/robbert-vdh/nih-plug";
const DEFAULT_EMAIL: &str = "info@example.com";
const DEFAULT_VST_ID: &str = "Exactly16Chars!!";
const DEFAULT_MIDI_CONFIG: &str = "None";
const DEFAULT_SUB_CATEGORY: &str = "Vst3SubCategory::Fx";

const DEFAULT_CLAP_ID: &str = "com.moist-plugins-gmbh.gain";
const DEFAULT_CLAP_DESCRIPTION: &str = "A smoothed gain parameter example plugin";

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
    // vst_data has to be initialized so that we have at least one export by default
    let mut vst_data = Some(Vst3Config {
        plugin_name: DEFAULT_NAME.to_owned(),
        vst_id: DEFAULT_VST_ID.to_owned(),
        sub_categories: DEFAULT_SUB_CATEGORY.to_owned(),
    });
    // clap/standalone exports are not defaults, so initialize them to None
    let mut clap_data = None;
    let mut standalone_config = None;

    let mut lib = LibConfig {
        plugin_name: DEFAULT_NAME.to_string(),
        vendor: DEFAULT_VENDOR.to_owned(),
        url: DEFAULT_URL.to_owned(),
        email: DEFAULT_EMAIL.to_owned(),
        midi_config: DEFAULT_MIDI_CONFIG.to_owned(),
    };

    let project_name: String = name.unwrap_or(
        input("What's your project named? (NOT the actual plugin name)")
            .placeholder("gain")
            .required(true)
            .interact()?,
    );

    let current_dir = current_dir().unwrap();
    let path = current_dir.join(&project_name);

    // TODO:
    // there is probably a way to refactor this to save on this indent,
    // i just didnt figure it out :(

    if !defaults {
        // get user input for basic plugin info
        intro("create-nih-plug-project").unwrap();

        let plugin_name: String = input("What's your plugin named?")
            .placeholder(DEFAULT_NAME)
            .default_input(DEFAULT_NAME)
            .interact()?;

        let vendor: String = input("Author?")
            .placeholder(DEFAULT_VENDOR)
            .default_input(DEFAULT_VENDOR)
            .interact()?;

        let url: String = input("URL?")
            .placeholder(DEFAULT_URL)
            .default_input(DEFAULT_URL)
            .interact()?;

        let email: String = input("Email?")
            .placeholder(DEFAULT_EMAIL)
            .default_input(DEFAULT_EMAIL)
            .interact()?;

        /*
         *
         * NOTE:
         * Audio config is not included here,
         * because some DAWs (Ableton, for example) do NOT support plugins with "weird" audio configs (0 outputs, etc.)
         *
         */

        let midi_config: String = select("MIDI Config?")
        .item("None", "None", "The plugin will not receive MIDI events.")
        .item("Basic", "Basic", "The plugin receives note on/off/choke events, pressure, and possibly standardized expression types.")
        .item(
            "MidiCCs",
            "Full",
            "The plugin receives full MIDI CCs as well as pitch bend information.",
        )
        .initial_value("None")
        .interact()?
        .to_owned();

        // beyond the basic info, we need to know which exports to set up
        let export_types = multiselect("Other export types?")
            .item(ExportType::Vst3, "VST3", "")
            .item(
                ExportType::Clap,
                "CLAP",
                "See https://cleveraudio.org/ for more info",
            )
            .item(
                ExportType::Standalone,
                "Standalone",
                "Creates a standalone application that can run outside of a DAW/VST host",
            )
            .initial_values(vec![ExportType::Vst3])
            .required(true)
            .interact()?;
        // update vst config
        vst_data = if export_types.contains(&ExportType::Vst3) {
            Some(configure_vst_export(&project_name)?)
        } else {
            // since VST is the default type,
            // if the user UN-selects VST, we need to consider that
            None
        };

        // handle CLAP configuration/code generation
        if export_types.contains(&ExportType::Clap) {
            clap_data = Some(configure_clap_export(&project_name)?);
        }
        // finally, standalone setup
        if export_types.contains(&ExportType::Standalone) {
            standalone_config = Some(StandaloneConfig {
                plugin_name: plugin_name.to_string(),
                project_name: project_name.to_string(),
            });
        }

        lib = LibConfig {
            plugin_name,
            vendor,
            url,
            email,
            midi_config,
        };
    }

    // now, create/modify files
    cargo_new(&project_name);
    write_to_toml(standalone_config.is_some(), &path)?;
    write_to_lib(&path, &lib, clap_data, vst_data)?;
    write_to_main(&path, standalone_config)?;

    if skip_first_build {
        return Ok(());
    }

    // finally, build the plugin
    let args = &["--release".to_owned()];
    set_current_dir(&path)?;
    build(&[project_name.clone()], args)?;
    bundle(&path.join("target"), &project_name, args, false)?;

    Ok(())
}
