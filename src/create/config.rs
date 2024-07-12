use super::{
    boilerplate::{ClapConfig, LibConfig, Vst3Config},
    ExportType,
};
use anyhow::Result;
use cliclack::{input, intro, multiselect, select};
pub const DEFAULT_NAME: &str = "Gain";
pub const DEFAULT_VENDOR: &str = "NIH-Plug";
pub const DEFAULT_URL: &str = "https://github.com/robbert-vdh/nih-plug";
pub const DEFAULT_EMAIL: &str = "info@example.com";
pub const DEFAULT_VST_ID: &str = "Exactly16Chars!!";
pub const DEFAULT_MIDI_CONFIG: &str = "None";
pub const DEFAULT_SUB_CATEGORY: &str = "Vst3SubCategory::Fx";

const DEFAULT_CLAP_ID: &str = "com.moist-plugins-gmbh.gain";
const DEFAULT_CLAP_DESCRIPTION: &str = "A smoothed gain parameter example plugin";

// TODO: choose a better name LMAO
pub fn configure_lib() -> Result<LibConfig> {
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
    Ok(LibConfig {
        plugin_name,
        vendor,
        url,
        email,
        midi_config,
    })
}

pub fn collect_export_types() -> Vec<ExportType> {
    multiselect("Other export types?")
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
        .interact()
        // TODO: fix/remove this unwrap
        .unwrap()
}

pub fn configure_vst_export(plugin_name: &str) -> Result<Vst3Config> {
    let vst_id: String = input("VST ID?")
        .placeholder(DEFAULT_VST_ID)
        .default_input(DEFAULT_VST_ID)
        .validate(|input: &String| {
            if input.len() != 16 {
                Err("VST3 ID must be exactly 16 characters.")
            } else {
                Ok(())
            }
        })
        .interact()?;
    let sub_categories = build_category_list(
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
    Ok(Vst3Config {
        plugin_name: plugin_name.to_string(),
        vst_id,
        sub_categories,
    })
}

pub fn configure_clap_export(plugin_name: &str) -> Result<ClapConfig> {
    // clap id
    let clap_id: String = input("CLAP ID?")
        .placeholder(DEFAULT_CLAP_ID)
        .default_input(DEFAULT_CLAP_ID)
        .interact()?;

    // clap description
    let clap_description: String = input("CLAP ID?")
        .placeholder(DEFAULT_CLAP_DESCRIPTION)
        .default_input(DEFAULT_CLAP_DESCRIPTION)
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

    Ok(ClapConfig {
        plugin_name: plugin_name.to_string(),
        clap_id,
        clap_description,
        clap_features: clap_features.to_owned(),
    })
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
