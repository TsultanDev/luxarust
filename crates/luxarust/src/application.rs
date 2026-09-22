pub struct Application();

impl Application {
    pub fn initialize() -> Result<Application, ()> {
        Ok(Application())
    }
    pub fn run(&mut self) -> Result<(), ()> {
        Ok(())
    }
}
