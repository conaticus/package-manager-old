use std::{path::Path, fs::{File, rename}};

use flate2::read::GzDecoder;
use tar::Archive;

pub fn dir_exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn extract_tarball(package_name: String, path: String, dest: String) {
    let zip = File::open(path).unwrap();
    let tarball = GzDecoder::new(zip);

    let mut archive = Archive::new(tarball);
    archive.unpack(dest.clone()).unwrap();
    rename(format!("{}/package", dest), format!("{}/{}", dest, package_name)).unwrap();
}