use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use crate::plugin::Plugin;
use crate::schedule::Schedule;
use crate::system::{SystemId, SystemSlot};

struct ResourceEntry {
    value: Box<dyn Any>,
    owner: Option<TypeId>,
}

pub struct App {
    plugins: HashSet<TypeId>,
    schedules: HashMap<Schedule, Vec<SystemSlot>>,
    resources: HashMap<TypeId, ResourceEntry>,
    current_plugin: Option<TypeId>,
    system_owners: HashMap<SystemId, TypeId>,
    removed_systems: HashSet<SystemId>,
    next_system_id: u64,
    frame: u64,
    exit_requested: bool,
}

impl App {
    pub fn new() -> App {
        App {
            plugins: HashSet::new(),
            schedules: HashMap::new(),
            resources: HashMap::new(),
            current_plugin: None,
            system_owners: HashMap::new(),
            removed_systems: HashSet::new(),
            next_system_id: 0,
            frame: 0,
            exit_requested: false,
        }
    }

    pub fn add_plugins<P: Plugin + 'static>(&mut self, plugin: P) -> &mut Self {
        let type_id = TypeId::of::<P>();
        if plugin.is_unique() && self.plugins.contains(&type_id) {
            println!(
                "[luxarust] plugin `{}` already added, skipping",
                plugin.name()
            );
            return self;
        }
        self.plugins.insert(type_id);
        let previous = self.current_plugin.replace(type_id);
        plugin.build(self);
        self.current_plugin = previous;
        self
    }

    pub fn is_plugin_added<P: 'static>(&self) -> bool {
        self.plugins.contains(&TypeId::of::<P>())
    }

    pub fn remove_plugins<P: Plugin + 'static>(&mut self) -> bool {
        let type_id = TypeId::of::<P>();
        if !self.plugins.remove(&type_id) {
            return false;
        }

        let removed: Vec<SystemId> = self
            .system_owners
            .iter()
            .filter(|(_, owner)| **owner == type_id)
            .map(|(system_id, _)| *system_id)
            .collect();
        for system_id in removed {
            self.removed_systems.insert(system_id);
            self.system_owners.remove(&system_id);
        }
        for list in self.schedules.values_mut() {
            list.retain(|slot| slot.owner != Some(type_id));
        }

        let resources: Vec<TypeId> = self
            .resources
            .iter()
            .filter(|(_, entry)| entry.owner == Some(type_id))
            .map(|(resource_type, _)| *resource_type)
            .collect();
        for resource_type in resources {
            self.resources.remove(&resource_type);
        }
        true
    }

    pub fn add_systems(
        &mut self,
        schedule: Schedule,
        system: impl FnMut(&mut App) + 'static,
    ) -> &mut Self {
        let system_id = SystemId(self.next_system_id);
        self.next_system_id += 1;
        if let Some(owner) = self.current_plugin {
            self.system_owners.insert(system_id, owner);
        }
        let slot = SystemSlot {
            id: system_id,
            owner: self.current_plugin,
            func: Box::new(system),
        };
        self.schedules.entry(schedule).or_default().push(slot);
        self
    }

    pub fn insert_resource<T: 'static>(&mut self, value: T) -> &mut Self {
        let entry = ResourceEntry {
            value: Box::new(value),
            owner: self.current_plugin,
        };
        self.resources.insert(TypeId::of::<T>(), entry);
        self
    }

    pub fn insert_persistent_resource<T: 'static>(&mut self, value: T) -> &mut Self {
        let entry = ResourceEntry {
            value: Box::new(value),
            owner: None,
        };
        self.resources.insert(TypeId::of::<T>(), entry);
        self
    }

    pub fn resource<T: 'static>(&self) -> Option<&T> {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|entry| entry.value.downcast_ref())
    }

    pub fn resource_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .and_then(|entry| entry.value.downcast_mut())
    }

    pub fn remove_resource<T: 'static>(&mut self) -> Option<T> {
        self.resources
            .remove(&TypeId::of::<T>())
            .and_then(|entry| entry.value.downcast().ok())
            .map(|boxed| *boxed)
    }

    pub fn frame(&self) -> u64 {
        self.frame
    }

    pub fn exit(&mut self) {
        self.exit_requested = true;
    }

    pub fn should_exit(&self) -> bool {
        self.exit_requested
    }

    #[allow(clippy::result_unit_err)]
    pub fn run(&mut self) -> Result<(), ()> {
        self.exit_requested = false;
        self.frame = 0;
        self.run_schedules(&[
            Schedule::First,
            Schedule::PreStartup,
            Schedule::Startup,
            Schedule::PostStartup,
        ]);

        while !self.exit_requested {
            self.run_schedules(&[
                Schedule::First,
                Schedule::PreUpdate,
                Schedule::Update,
                Schedule::PostUpdate,
                Schedule::Last,
            ]);
            std::thread::sleep(Duration::from_millis(16));
            self.frame += 1;
        }
        Ok(())
    }

    fn run_schedules(&mut self, order: &[Schedule]) {
        for schedule in order {
            self.run_schedule(*schedule);
        }
    }

    fn run_schedule(&mut self, schedule: Schedule) {
        let Some(list) = self.schedules.get_mut(&schedule) else {
            return;
        };
        let mut systems = std::mem::take(list);
        systems.retain(|slot| !self.removed_systems.contains(&slot.id));

        let mut kept = Vec::with_capacity(systems.len());
        for mut slot in systems {
            if self.removed_systems.contains(&slot.id) {
                continue;
            }
            let previous = self.current_plugin;
            self.current_plugin = slot.owner;
            (slot.func)(&mut *self);
            self.current_plugin = previous;
            kept.push(slot);
        }

        if let Some(list) = self.schedules.get_mut(&schedule) {
            let mut added = std::mem::take(list);
            added.retain(|slot| !self.removed_systems.contains(&slot.id));
            kept.extend(added);
            *list = kept;
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn unique_plugin_built_only_once() {
        #[derive(Clone)]
        struct CountingPlugin {
            builds: Rc<Cell<usize>>,
        }

        impl Plugin for CountingPlugin {
            fn build(&self, _app: &mut App) {
                self.builds.set(self.builds.get() + 1);
            }
        }

        let builds = Rc::new(Cell::new(0));
        let mut app = App::new();
        app.add_plugins(CountingPlugin {
            builds: builds.clone(),
        });
        app.add_plugins(CountingPlugin {
            builds: builds.clone(),
        });
        assert_eq!(builds.get(), 1);
        assert!(app.is_plugin_added::<CountingPlugin>());
    }

    #[test]
    fn startup_runs_before_update() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut app = App::new();
        let log_startup = log.clone();
        app.add_systems(Schedule::Startup, move |_app: &mut App| {
            log_startup.borrow_mut().push("startup");
        });
        let log_update = log.clone();
        app.add_systems(Schedule::Update, move |app: &mut App| {
            log_update.borrow_mut().push("update");
            app.exit();
        });
        app.run().unwrap();
        assert_eq!(log.borrow().as_slice(), ["startup", "update"]);
    }

    #[test]
    fn systems_run_in_registration_order() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut app = App::new();
        for label in ["a", "b", "c", "exit"] {
            let log_here = log.clone();
            app.add_systems(Schedule::Update, move |app: &mut App| {
                log_here.borrow_mut().push(label);
                if label == "exit" {
                    app.exit();
                }
            });
        }
        app.run().unwrap();
        assert_eq!(log.borrow().as_slice(), ["a", "b", "c", "exit"]);
    }

    #[test]
    fn plugin_registers_systems_and_resources() {
        struct DemoPlugin;

        impl Plugin for DemoPlugin {
            fn build(&self, app: &mut App) {
                app.insert_resource(42u32);
                app.add_systems(Schedule::Startup, demo_startup);
                app.add_systems(Schedule::Update, demo_update);
            }
        }

        fn demo_startup(app: &mut App) {
            app.insert_resource(String::from("started"));
        }

        fn demo_update(app: &mut App) {
            app.exit();
        }

        let mut app = App::new();
        app.add_plugins(DemoPlugin);
        assert!(app.is_plugin_added::<DemoPlugin>());
        assert_eq!(app.resource::<u32>(), Some(&42));
        assert!(app.resource::<String>().is_none());
        app.run().unwrap();
        assert_eq!(app.resource::<String>(), Some(&"started".to_string()));
    }

    #[test]
    fn remove_plugins_removes_owned_systems_and_resources() {
        struct Owned;

        impl Plugin for Owned {
            fn build(&self, app: &mut App) {
                app.insert_resource(String::from("owned"));
                app.add_systems(Schedule::Update, |_app: &mut App| {});
            }
        }

        struct Unowned;

        impl Plugin for Unowned {
            fn build(&self, app: &mut App) {
                app.insert_resource(7u8);
            }
        }

        let mut app = App::new();
        app.add_plugins(Owned);
        app.add_plugins(Unowned);

        assert!(app.resource::<String>().is_some());
        assert!(app.remove_plugins::<Owned>());
        assert!(app.resource::<String>().is_none());
        assert!(app.resource::<u8>().is_some());
        assert!(!app.is_plugin_added::<Owned>());
        assert!(!app.remove_plugins::<Owned>());

        app.add_plugins(Owned);
        assert!(app.is_plugin_added::<Owned>());
        assert!(app.resource::<String>().is_some());
    }

    #[test]
    fn resources_roundtrip() {
        let mut app = App::new();
        app.insert_resource(42u32);
        assert_eq!(app.resource::<u32>(), Some(&42));
        *app.resource_mut::<u32>().unwrap() += 1;
        assert_eq!(app.remove_resource::<u32>(), Some(43));
        assert!(app.resource::<u32>().is_none());
    }

    #[test]
    fn persistent_resource_survives_plugin_removal() {
        struct Host;

        impl Plugin for Host {
            fn build(&self, app: &mut App) {
                app.insert_persistent_resource(9i16);
            }
        }

        struct Removable;

        impl Plugin for Removable {
            fn build(&self, app: &mut App) {
                app.insert_resource(true);
            }
        }

        let mut app = App::new();
        app.add_plugins(Host);
        app.add_plugins(Removable);
        assert!(app.remove_plugins::<Removable>());
        assert!(app.resource::<bool>().is_none());
        assert_eq!(app.resource::<i16>(), Some(&9));
        assert!(app.remove_plugins::<Host>());
        assert_eq!(app.resource::<i16>(), Some(&9));
    }

    #[test]
    fn runtime_plugin_swap_is_safe() {
        struct First;

        impl Plugin for First {
            fn build(&self, app: &mut App) {
                app.insert_resource(1u32);
                app.add_systems(Schedule::Update, |app: &mut App| {
                    if app.resource::<u32>().is_some_and(|value| *value == 1) {
                        app.remove_plugins::<First>();
                        app.add_plugins(Second);
                        app.exit();
                    }
                });
            }
        }

        struct Second;

        impl Plugin for Second {
            fn build(&self, app: &mut App) {
                app.insert_resource(2u32);
            }
        }

        let mut app = App::new();
        app.add_plugins(First);
        app.run().unwrap();
        assert!(app.is_plugin_added::<Second>());
        assert!(!app.is_plugin_added::<First>());
        assert_eq!(app.resource::<u32>(), Some(&2));
    }
}