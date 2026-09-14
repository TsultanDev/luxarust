pub enum Exception {
    ModuleLoadFailed,
}

pub struct LoaderCreateConfig {
    app_name: String,
    app_version: (u32, u32, u32),
    api_version: (u32, u32, u32),
}

pub struct Loader {
    entry: ash::Entry,
    instance: ash::Instance,
}
impl Loader {
    pub fn load(config_file: &str) -> Result<Loader, Exception> {
        unsafe {
            let entry = ash::Entry::load();
            let entry = match entry {
                Ok(e) => e,
                Err(_) => return Err(Exception::ModuleLoadFailed),
            };

            Ok(())
        }
    }
}
impl Drop for Loader {
    fn drop(&mut self) {}
}
