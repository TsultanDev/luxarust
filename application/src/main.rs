use std::path::PathBuf;

use luxarust::Application;

fn main() {
    println!("Hello from Luxarust!");
    let config_arg = std::env::args().nth(1);
    let config_file = find_config(config_arg);
    println!("Loaded config: {config_file}");

    let mut app = match Application::load(&config_file) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("Failed to load config '{config_file}'. Error {e:?}");
            return;
        }
    };

    match app.initialize() {
        Ok(_) => println!("App Exit Successfully!!"),
        Err(e) => eprintln!("App Exit Unsuccessfully!!. Error {e:?}"),
    }
}

fn find_config(explicit: Option<String>) -> String {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(path) = explicit {
        candidates.push(PathBuf::from(path));
    }
    candidates.push(PathBuf::from("configuration.toml"));
    candidates.push(PathBuf::from("application/configuration.toml"));
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("configuration.toml"));
        }
    }

    for candidate in candidates {
        if candidate.exists() {
            return candidate.to_string_lossy().into_owned();
        }
    }
    "configuration.toml".to_string()
}