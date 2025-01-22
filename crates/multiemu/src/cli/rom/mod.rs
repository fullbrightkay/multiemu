use crate::rom::{id::RomId, system::GameSystem};
use clap::{Parser, Subcommand, ValueEnum};
use std::{error::Error, path::PathBuf, str::FromStr};

pub mod import;
pub mod run;

#[derive(Debug, Clone)]
pub enum RomSpecification {
    Id(RomId),
    Path(PathBuf),
}

impl FromStr for RomSpecification {
    type Err = Box<dyn Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path = PathBuf::from(s);
        if path.is_file() {
            return Ok(RomSpecification::Path(path));
        }

        // If it's not a valid path, try to parse as a RomId
        match RomId::from_str(s) {
            Ok(id) => Ok(RomSpecification::Id(id)),
            Err(err) => Err(err.into()),
        }
    }
}

#[derive(Clone, Debug, Subcommand)]
pub enum RomAction {
    Import {
        #[clap(required=true, num_args=1..)]
        paths: Vec<PathBuf>,
        #[clap(short, long)]
        symlink: bool,
    },
    Run {
        roms: Vec<RomSpecification>,
        #[clap(short, long)]
        forced_system: Option<GameSystem>,
    },
}
