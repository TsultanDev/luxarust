use crate::app::App;

pub trait Plugin {
    fn build(&self, app: &mut App);

    fn is_unique(&self) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}