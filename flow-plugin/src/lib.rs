pub mod descriptor;
pub mod plugin;
pub mod manager;
pub mod loader;
pub mod ffi;

pub use descriptor::PluginDescriptor;
pub use plugin::{Plugin, PluginWrapper, PluginState};
pub use manager::{PluginManager, DefaultPluginManager};
pub use loader::{PluginLoader, DynamicLibraryLoader, DirectoryPluginLoader};
