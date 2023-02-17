use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone)]
pub struct PackageRepository {
    pub r#type: Option<String>,
    pub url: String,
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum RepositoryResult {
    Str(String),
    Full(PackageRepository), 
}

#[derive(Deserialize, Clone)]
pub struct PackageVersionDist {
    pub tarball: String,
}

#[derive(Deserialize, Clone)]
pub struct PackageVersion {
    pub name: Option<String>,
    pub version: String,
    pub repository: Option<RepositoryResult>,
    pub dependencies: Option<HashMap<String, String>>,
    #[serde(alias = "devDependencies")]
    pub dev_dependencies: Option<HashMap<String, String>>,
    pub main: Option<String>,
    pub dist: PackageVersionDist,
}

#[derive(Deserialize, Clone)]
pub struct DistTags {
    pub latest: String,
}

#[derive(Deserialize, Clone)]
pub struct Package {
    pub name: Option<String>,
    #[serde(alias = "dist-tags")]
    pub dist_tags: DistTags,
    pub versions: HashMap<String, PackageVersion>,
}