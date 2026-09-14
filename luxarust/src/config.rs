use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Exception {
    ConfigReadFailed,
    ConfigParseFailed,
    EventLoopCreatedFailed,
    WindowCreateFailed,
    InternalAppError,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub name: String,
    pub version: Version,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
    pub resizable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub application: ApplicationConfig,
    pub window: WindowConfig,
}

impl Config {
    pub fn load(config_file: &str) -> Result<Config, Exception> {
        let content =
            std::fs::read_to_string(config_file).map_err(|_| Exception::ConfigReadFailed)?;
        toml::from_str(&content).map_err(|_| Exception::ConfigParseFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config_toml() {
        let toml_str = r#"
            [application]
            name = "Luxarust"
            version = { major = 0, minor = 1, patch = 0 }

            [window]
            title = "Luxarust"
            width = 1280
            height = 720
            maximized = false
            resizable = true
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.application.name, "Luxarust");
        assert_eq!(config.window.title, "Luxarust");
        assert_eq!(config.window.width, 1280);
        assert_eq!(config.window.height, 720);
        assert!(config.window.resizable);
        assert!(!config.window.maximized);
    }
}