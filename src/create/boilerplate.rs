// This module contains boilerplate structs,

use super::config::{
    DEFAULT_EMAIL, DEFAULT_MIDI_CONFIG, DEFAULT_NAME, DEFAULT_SUB_CATEGORY, DEFAULT_URL,
    DEFAULT_VENDOR, DEFAULT_VST_ID,
};

#[derive(boilerplate::Boilerplate)]
#[boilerplate(filename = "lib.txt")]
#[derive(Clone)]
pub struct LibConfig {
    pub plugin_name: String,
    pub vendor: String,
    pub url: String,
    pub email: String,
    pub midi_config: String,
}

impl Default for LibConfig {
    fn default() -> Self {
        Self {
            plugin_name: DEFAULT_NAME.to_string(),
            vendor: DEFAULT_VENDOR.to_string(),
            url: DEFAULT_URL.to_string(),
            email: DEFAULT_EMAIL.to_string(),
            midi_config: DEFAULT_MIDI_CONFIG.to_string(),
        }
    }
}

#[derive(boilerplate::Boilerplate)]
#[boilerplate(filename = "main.txt")]
pub struct StandaloneConfig {
    pub plugin_name: String,
    pub project_name: String,
}

#[derive(boilerplate::Boilerplate)]
#[boilerplate(filename = "clap.txt")]
pub struct ClapConfig {
    pub plugin_name: String,
    pub clap_id: String,
    pub clap_description: String,
    pub clap_features: String,
}

#[derive(boilerplate::Boilerplate)]
#[boilerplate(filename = "vst3.txt")]
pub struct Vst3Config {
    pub plugin_name: String,
    pub vst_id: String,
    pub sub_categories: String,
}

impl Default for Vst3Config {
    fn default() -> Self {
        Self {
            plugin_name: DEFAULT_NAME.to_string(),
            vst_id: DEFAULT_VST_ID.to_string(),
            sub_categories: DEFAULT_SUB_CATEGORY.to_string(),
        }
    }
}
