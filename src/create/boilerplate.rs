// This module contains boilerplate structs,

#[derive(boilerplate::Boilerplate)]
#[boilerplate(filename = "lib.txt")]
pub struct LibConfig {
    pub plugin_name: String,
    pub vendor: String,
    pub url: String,
    pub email: String,
    pub midi_config: String,
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
