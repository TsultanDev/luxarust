use luxarust::Application;

fn main() {
    let app = Application::initialize().unwrap().run();
    println!("Hello, world!");
}
