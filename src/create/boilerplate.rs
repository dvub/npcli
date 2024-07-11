// This module contains boilerplate structs,

#[derive(boilerplate::Boilerplate)]
pub struct LibTxt {
    pub plugin_name: String,
    pub vendor: String,
    pub url: String,
    pub email: String,
    pub midi_config: String,
}

#[derive(boilerplate::Boilerplate)]
pub struct MainTxt {
    pub plugin_name: String,
    pub project_name: String,
}

#[derive(boilerplate::Boilerplate)]
pub struct ClapTxt {
    pub plugin_name: String,
    pub clap_id: String,
    pub clap_description: String,
    pub clap_features: String,
}

#[derive(boilerplate::Boilerplate)]
pub struct Vst3Txt {
    pub plugin_name: String,
    pub vst_id: String,
    pub sub_categories: String,
}
