use luxarust::prelude::*;
use luxarust_window::WindowPlugin;

struct UnplugDemo;

impl Plugin for UnplugDemo {
    fn build(&self, app: &mut App) {
        app.add_systems(Schedule::Update, demo_tick);
    }

    fn name(&self) -> &'static str {
        "UnplugDemo"
    }
}

fn demo_tick(app: &mut App) {
    match app.frame() {
        0 => println!("[demo] WindowPlugin attached, unplugging in ~5s..."),
        300 => {
            println!("[demo] unplugging WindowPlugin...");
            app.remove_plugins::<WindowPlugin>();
        }
        600 => {
            println!("[demo] re-plugging WindowPlugin...");
            app.add_plugins(WindowPlugin::default());
        }
        _ => {}
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(UnplugDemo);
    app.add_plugins(WindowPlugin::default());

    app.run().expect("application run failed");

    println!(
        "[demo] exited; WindowPlugin present: {}",
        app.is_plugin_added::<WindowPlugin>()
    );
}