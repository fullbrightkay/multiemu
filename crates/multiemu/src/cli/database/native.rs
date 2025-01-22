use clap::Subcommand;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{error::Error, path::PathBuf};
use crate::{config::GLOBAL_CONFIG, rom::manager::RomManager};

#[derive(Clone, Debug, Subcommand)]
pub enum NativeAction {
    Import {
        #[clap(required=true, num_args=1..)]
        paths: Vec<PathBuf>,
    },
}

pub fn database_native_import(paths: Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    let global_config_guard = GLOBAL_CONFIG.try_read()?;
    let rom_manager = RomManager::new(Some(&global_config_guard.database_file))?;

    paths
        .into_par_iter()
        .try_for_each(|path| rom_manager.load_database(path))
        .map_err(|err| err as Box<dyn Error>)?;

    Ok(())
}
