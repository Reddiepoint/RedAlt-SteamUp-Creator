use serde::Deserialize;

#[derive(Clone, Default, Deserialize)]
pub struct Changelog {
    pub depot_id: String,
    pub manifest: String,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub modified: Vec<String>,
}
#[derive(Clone, Default, Deserialize)]
pub struct Changes {
    pub app_name: String,
    pub app_id: String,
    pub initial_build: String,
    pub final_build: String,
    pub changelogs: Vec<Changelog>,
}