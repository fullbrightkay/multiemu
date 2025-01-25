use crate::machine::ComponentBuilder;
use crate::memory::MemoryTranslationTable;
use downcast_rs::DowncastSync;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

pub mod display;
pub mod input;
pub mod memory;
pub mod schedulable;

// Basic supertrait for all components
pub trait Component: Any + Debug + Send + Sync + DowncastSync {
    fn reset(&self) {}
    fn save_snapshot(&self) -> rmpv::Value {
        rmpv::Value::Nil
    }
    fn load_snapshot(&self, _snapshot: rmpv::Value) {}
    fn set_memory_translation_table(&self, _memory_translation_table: Arc<MemoryTranslationTable>) {
    }
}

// An initializable component
pub trait FromConfig: Component + Sized {
    type Config: Debug;

    /// Make a new component from the config
    fn from_config(component_builder: &mut ComponentBuilder<Self>, config: Self::Config);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(pub u16);
