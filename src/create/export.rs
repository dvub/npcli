use super::{
    boilerplate::{ClapConfig, Vst3Config},
    DEFAULT_CLAP_DESCRIPTION, DEFAULT_CLAP_ID, DEFAULT_VST_ID,
};
use anyhow::Result;
use cliclack::{input, multiselect, select};

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
