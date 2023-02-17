use core::panic;
use std::{error::Error, fs::{File}, io::{self, Cursor}};
use serde::de::DeserializeOwned;
use super::types::Package;

pub async fn get<T: DeserializeOwned>(url: String) -> Result<T, Box<dyn Error>> {
    let res = reqwest::get(url).await?.json::<T>().await?;
    Ok(res)
}

pub async fn download_file(package_name: String, url: String, directory: String) -> String {
    let res = match reqwest::get(url.clone()).await {
        Ok(resp) => resp,
        Err(_) => panic!("Could not download file from '{}'", url),
    };


    let file_path = format!("{}/{}", directory, package_name);

    let mut out = File::create(file_path.clone()).unwrap();
    let mut content = Cursor::new(res.bytes().await.unwrap());

    io::copy(&mut content, &mut out).unwrap();

    file_path
}

// Util queryies

pub async fn get_package(package_name: String) -> Package {
    let result = get::<Package>(format!("https://registry.npmjs.com/{}", package_name)).await;

    match result {
        Ok(pak) => pak,
        Err(e) => panic!("Could not find package '{}'.\nError: {}", package_name, e),
    }
}