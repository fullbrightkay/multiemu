use crate::{
    component::input::EmulatedGamepadTypeId,
    input::{
        hotkey::{Hotkey, DEFAULT_HOTKEYS},
        Input,
    },
    rom::system::GameSystem,
};
use indexmap::IndexMap;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use serde_with::serde_as;
use serde_with::DefaultOnError;
use std::{
    collections::BTreeSet,
    sync::{LazyLock, RwLock},
};
use std::{
    fs::{create_dir_all, File},
    ops::Deref,
    path::PathBuf,
};
use strum::{Display, EnumIter};

/// The directory where we store our runtime files is platform specific
#[cfg(platform_desktop)]
pub static STORAGE_DIRECTORY: LazyLock<PathBuf> =
    LazyLock::new(|| dirs::data_dir().unwrap().join("multiemu"));
#[cfg(platform_3ds)]
pub static STORAGE_DIRECTORY: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from("sdmc:/multiemu"));

pub static CONFIG_LOCATION: LazyLock<PathBuf> =
    LazyLock::new(|| STORAGE_DIRECTORY.join("config.ron"));

#[derive(Serialize, Deserialize, Debug, Clone, Copy, EnumIter, Display, PartialEq, Eq)]
pub enum GraphicsSettings {
    Software,
    #[cfg(graphics_vulkan)]
    Vulkan,
}

#[allow(clippy::derivable_impls)]
impl Default for GraphicsSettings {
    fn default() -> Self {
        // TODO: Once the ui rendering backend for vulkan is done, enable it here
        GraphicsSettings::Software
    }
}

#[serde_as]
#[serde_inline_default]
#[derive(Serialize, Deserialize, Debug)]
pub struct GlobalConfig {
    #[serde(default)]
    pub gamepad_configs:
        IndexMap<GameSystem, IndexMap<EmulatedGamepadTypeId, IndexMap<Input, Input>>>,
    #[serde_inline_default(DEFAULT_HOTKEYS.clone())]
    pub hotkeys: IndexMap<BTreeSet<Input>, Hotkey>,
    #[serde(default)]
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub graphics_setting: GraphicsSettings,
    #[serde_inline_default(true)]
    pub vsync: bool,
    #[serde_inline_default(STORAGE_DIRECTORY.clone())]
    pub file_browser_home: PathBuf,
    #[serde_inline_default(STORAGE_DIRECTORY.join("log"))]
    pub log_location: PathBuf,
    #[serde_inline_default(STORAGE_DIRECTORY.join("database"))]
    pub database_file: PathBuf,
    #[serde_inline_default(STORAGE_DIRECTORY.join("saves"))]
    pub save_directory: PathBuf,
    #[serde_inline_default(STORAGE_DIRECTORY.join("snapshot"))]
    pub snapshot_directory: PathBuf,
    #[serde_inline_default(STORAGE_DIRECTORY.join("roms"))]
    pub roms_directory: PathBuf,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            gamepad_configs: Default::default(),
            hotkeys: DEFAULT_HOTKEYS.clone(),
            graphics_setting: GraphicsSettings::default(),
            vsync: true,
            file_browser_home: STORAGE_DIRECTORY.clone(),
            log_location: STORAGE_DIRECTORY.join("log"),
            database_file: STORAGE_DIRECTORY.join("database"),
            save_directory: STORAGE_DIRECTORY.join("saves"),
            snapshot_directory: STORAGE_DIRECTORY.join("snapshot"),
            roms_directory: STORAGE_DIRECTORY.join("roms"),
        }
    }
}

impl GlobalConfig {
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        create_dir_all(STORAGE_DIRECTORY.deref())?;
        let config_file = File::create(CONFIG_LOCATION.deref())?;
        ron::ser::to_writer_pretty(config_file, self, PrettyConfig::default())?;

        Ok(())
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_file = File::open(CONFIG_LOCATION.deref())?;
        let config = ron::de::from_reader(config_file)?;

        Ok(config)
    }
}

/// FIXME: This is a mutable singleton out of lazyness
pub static GLOBAL_CONFIG: LazyLock<RwLock<GlobalConfig>> =
    LazyLock::new(|| RwLock::new(GlobalConfig::load().unwrap_or_default()));
