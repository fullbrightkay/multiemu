use dashmap::DashMap;

use super::{id::RomId, info::RomInfo};
use std::{
    collections::HashMap,
    error::Error,
    fs::{create_dir_all, read_dir, File},
    path::{Path, PathBuf},
    sync::LazyLock,
};

static DATABASE_MODELS: LazyLock<native_db::Models> = LazyLock::new(|| {
    let mut models = native_db::Models::new();
    models.define::<RomInfo>().unwrap();
    models
});

pub struct RomManager {
    pub rom_information: native_db::Database<'static>,
    pub rom_paths: DashMap<RomId, PathBuf>,
}

impl RomManager {
    /// Opens and loads the default database
    pub fn new(database: Option<&Path>) -> Result<Self, Box<dyn Error>> {
        let rom_information = if let Some(path) = database {
            let _ = create_dir_all(path.parent().unwrap());

            native_db::Builder::new().create(&DATABASE_MODELS, path)?
        } else {
            native_db::Builder::new().create_in_memory(&DATABASE_MODELS)?
        };

        Ok(Self {
            rom_information,
            rom_paths: DashMap::new(),
        })
    }

    pub fn load_database(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
        let path = path.as_ref();

        if !path.is_file() {
            return Err("Path is not a file".into());
        }

        let database = native_db::Builder::new().open(&DATABASE_MODELS, path)?;
        let external_database_transaction = database.r_transaction()?;
        let internal_database_transaction = self.rom_information.rw_transaction()?;

        for item in (external_database_transaction
            .scan()
            .primary::<RomInfo>()?
            .all()?)
        .flatten()
        {
            internal_database_transaction.upsert(item)?;
        }

        internal_database_transaction.commit()?;

        Ok(())
    }

    pub fn load_roms(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
        let path = path.as_ref();
        let roms = read_dir(path)?;

        for rom in roms {
            let rom = rom?;
            let path = rom.path();

            if !path.is_file() {
                continue;
            }

            let path_name: RomId = path
                .canonicalize()?
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .parse()?;

            self.rom_paths.insert(path_name, path);
        }

        Ok(())
    }

    pub fn load_rom_paths_verified(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<HashMap<RomId, PathBuf>, Box<dyn Error>> {
        let path = path.as_ref();
        let roms = read_dir(path)?;

        let mut incorrect_roms = HashMap::new();

        for rom in roms {
            let rom = rom?;
            let path = rom.path();

            if !path.is_file() {
                continue;
            }

            let expected_hash = path.file_name().unwrap().to_str().unwrap().parse()?;

            let mut file = File::open(&path)?;
            let hash = RomId::from_read(&mut file);

            if hash != expected_hash {
                incorrect_roms.insert(hash, path);
            } else {
                self.rom_paths.insert(hash, path);
            }
        }

        Ok(incorrect_roms)
    }

    /// Components should use this function to load roms for themselves
    pub fn open(&self, id: RomId, requirement: RomRequirement) -> Option<File> {
        if let Some(path) = self.rom_paths.get(&id) {
            return File::open(path.value()).ok();
        }

        match requirement {
            RomRequirement::Sometimes => {
                tracing::warn!(
                    "Could not find ROM {} for machine, machine will continue in a degraded state",
                    id
                );
            }
            RomRequirement::Optional => {
                tracing::info!(
                    "Could not find ROM {} for machine, but it's optional for runtime",
                    id
                );
            }
            RomRequirement::Required => {
                tracing::error!("ROM {} is required for machine, but not found", id);
            }
        }

        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RomRequirement {
    /// Ok to boot machine without this ROM but runtime failure can occur without it
    Sometimes,
    /// Machine will boot emulating this ROM
    Optional,
    /// Machine can not boot without this ROM
    Required,
}
