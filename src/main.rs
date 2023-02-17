mod common;
mod operations;

use std::env;
use operations::install::install;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let operation = &args[1];
    let package_query = &args[2];

    match operation.to_lowercase().as_str() {
        "install" => install(package_query).await,
        _ => panic!("Operation '{}' does not exist.", operation),
    }
}