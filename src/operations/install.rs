use std::{collections::HashMap, fs::create_dir};

use crate::common::{types::{Package, PackageVersion}, http::{get_package, download_file}, fs::{dir_exists, extract_tarball}};
use async_recursion::async_recursion;

const MODULES_DIR: &str = "./node_modules";
const TEMP_DIR: &str = "./node_modules/temp";

pub async fn install(package_query: &String) {
    if !dir_exists(MODULES_DIR) {
        create_dir(MODULES_DIR).unwrap();
    }

    if !dir_exists(TEMP_DIR) {
        create_dir(TEMP_DIR).unwrap();
    }
    
    println!("Searching registry for '{}'", package_query);

    let (package_name, version) = get_version(package_query);

    let package = get_package(package_name.into()).await;
    println!("Found package '{}'", package_name);

    let dependency_list = get_dependency_list(&package, version).await;
    let mut progression = 0;

    for package in dependency_list.values() {
        progression += 1;
        if package.repository.is_none() { continue };

        let zip_path = download_file(stringify_opt(&package.name), package.dist.tarball.clone(), TEMP_DIR.into()).await;
        extract_tarball(stringify_opt(&package.name), zip_path, MODULES_DIR.into());

        let progress_percent = ((progression as f32 / dependency_list.len() as f32) * 100.) as u8;
        println!("Downloaded {}@{} - {}% ({}/{})", stringify_opt(&package.name), package.version, progress_percent, progression, dependency_list.len());
    }
}

fn get_version(package_query: &String) -> (&str, Option<&str>) {
    let mut query_split = package_query.split("@");
    let package_name = query_split.next().unwrap();
    
    (package_name, query_split.next())
}

fn get_version_iterations(version: &str) -> (String, String, String) {
    if !version.contains(".") {
        return ("0".into(), "0".into(), "0".into());
    }

    let mut version_spl = version.split(".");
    let major = version_spl.next().unwrap();
    let minor = version_spl.next().unwrap();
    let patch = version_spl.next().unwrap();

    (major.into(), minor.into(), patch.into())
}

fn higher_version(a: &String, b: &String) -> Option<String> {
    let (a_major, a_minor, a_patch) = get_version_iterations(a);
    let (b_major, b_minor, b_patch) = get_version_iterations(b);

    let a_str = Some(a.clone());
    let b_str = Some(b.clone());

    if a_major > b_major {
        return a_str;
    } else if b_major > a_major {
        return b_str;
    } else {
        if a_minor > b_minor {
            return a_str;
        } else if b_minor > a_minor {
            return b_str;
        } else {
            if a_patch > b_patch {
                return a_str;
            } else if b_patch > a_patch {
                return b_str;
            } else {
                return None;
            }
        }
    }
}

/// Information about version format: https://stackoverflow.com/questions/22343224/whats-the-difference-between-tilde-and-caret-in-package-json
fn parse_version(package: &Package, version: &str) -> String {
    if version.starts_with("~") { // ~version
        let filtered: Vec<_> = package.versions.keys().filter(|&pkg_ver| {
            let (major, minor, _) = get_version_iterations(&version[1..]);
            let formatted_filter = format!("{}.{}.", major, minor);

            pkg_ver.starts_with(formatted_filter.as_str())
        }).collect();

        return filtered.last().unwrap().to_string();
    } else if version.starts_with("^")  { // ^version
        let filtered: Vec<_> = package.versions.keys().filter(|&pkg_ver| {
            let (major, _, _) = get_version_iterations(&version[1..]);
            let formatted_filter = format!("{}.", major);

            pkg_ver.starts_with(formatted_filter.as_str())
        }).collect();

        return filtered.last().unwrap().to_string();
    } else if version.starts_with(">")  { // >=version, >version
        let arr: Vec<_> = package.versions.keys().collect();
        return arr.last().unwrap().to_string();
    } else if version.starts_with("<=") { // <= version
        return String::from(&version[2..]);
    } else if version.starts_with("<") { // <version
        let mut highest_version = &String::new();
        for pkg_ver in package.versions.keys() {
            if pkg_ver.as_str() == &version[1..] { break }

            let higher_version = higher_version(highest_version, pkg_ver);
            if let Some(higher_version) = higher_version {
                if &higher_version == pkg_ver { highest_version = pkg_ver }
            }
        }

        return highest_version.clone();
    } else if version.ends_with("x") { // 1.2.x
        let filtered: Vec<_> = package.versions.keys().filter(|&pkg_ver| {
            let (major, minor, _) = get_version_iterations(&version[..1]);
            let formatted_filter = format!("{}.{}.", major, minor);

            pkg_ver.starts_with(formatted_filter.as_str())
        }).collect();

        let last = filtered.last();
        return match last {
            Some(&last) => last.to_string(),
            None => {
                let arr: Vec<_> = package.versions.keys().collect();
                arr.last().unwrap().to_string()
            }
        }
    } else { // *, latest, version
        let arr: Vec<_> = package.versions.keys().collect();
        return arr.last().unwrap().to_string();
    }
}

fn get_version_data(package: &Package, version: Option<&str>) -> PackageVersion {
    if version.is_none() {
        return package.versions.get(package.dist_tags.latest.as_str()).unwrap().clone();
    }

    let version = parse_version(package, version.unwrap());

    let version_opt = package.versions.get(version.as_str());
    match version_opt {
        Some(v) => v.clone(),
        None => panic!("Could not find version {} for package '{}'", version, stringify_opt(&package.name)),
    }
}

fn stringify_opt(opt: &Option<String>) -> String {
    match opt {
        Some(val) => val.clone(),
        None => String::from("unidentified"),
    }
}

#[async_recursion]
async fn get_dependency_list(package: &Package, version: Option<&'async_recursion str>) -> HashMap<String, PackageVersion> {
    let mut dependency_list: HashMap<String, PackageVersion> = HashMap::new();
    let version_data = get_version_data(package, version);
    println!("Searching dependencies for {}@{}", stringify_opt(&package.name), version_data.version);

    let mut dependencies: HashMap<String, String> = HashMap::new();
    if let Some(deps) = version_data.dependencies {
        dependencies.extend(deps);
    }

    // if let Some(deps) = version_data.dev_dependencies { NOTE: dev dependencies should be considered
    //     dependencies.extend(deps);
    // }

    for (package_name, version_raw) in dependencies {
        let dependency = get_package(package_name.clone()).await;
        let version_data = get_version_data(&dependency, Some(version_raw.as_str()));

        if dependency_list.get_mut(&package_name).is_none() {
            let list = get_dependency_list(&dependency, Some(version_data.version.as_str())).await;
            dependency_list.insert(package_name.clone(), version_data);
            dependency_list.extend(list);
        }
    }


    dependency_list
}