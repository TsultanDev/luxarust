#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Schedule {
    First,
    PreStartup,
    Startup,
    PostStartup,
    PreUpdate,
    Update,
    PostUpdate,
    Last,
}