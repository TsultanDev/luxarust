use luxarust::Application;

fn main() {
    println!("Hello from Luxarust!");
    let app = Application::default().initialize();
    match app {
        Ok(_) => println!("App Exit Successfully!!"),
        Err(e) => eprintln!("App Exit Unsuccessfully!!. Error {:?}", e),
    }
}
