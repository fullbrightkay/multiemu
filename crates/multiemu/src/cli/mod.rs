use crate::rom::{id::RomId, system::GameSystem};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

pub mod import_known_roms;
pub mod import_native_database;
pub mod import_nointro_database;
pub mod import_rom_manually;
pub mod run_external_rom;
// pub mod run_rom;

#[derive(ValueEnum, Clone, Debug)]
pub enum DatabaseType {
    Native,
    Nointro,
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub action: Option<CliAction>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum CliAction {
    ImportDatabase {
        #[clap(short, long)]
        database_type: DatabaseType,
        #[arg(required=true, num_args=1..)]
        path: Vec<PathBuf>,
    },
    ImportRomManually {
        #[clap(short, long)]
        symlink: bool,
        #[clap(short, long)]
        game_system: GameSystem,
        #[clap(short, long)]
        name: Option<String>,
        #[clap(short, long)]
        path: PathBuf,
    },
    ImportKnownRoms {
        #[clap(short, long)]
        symlink: bool,
        #[arg(required=true, num_args=1..)]
        paths: Vec<PathBuf>,
    },
    VerifyRoms {
        #[clap(short, long)]
        unknown_discard: bool,
        #[clap(short, long)]
        incorrect_discard: bool,
    },
    Run {
        #[clap(short, long)]
        force_game_system: Option<GameSystem>,
        #[arg(required=true, num_args=1..)]
        rom: Vec<RomId>,
    },
    RunExternal {
        #[clap(short, long)]
        force_game_system: Option<GameSystem>,
        #[arg(required=true, num_args=1..)]
        rom: Vec<PathBuf>,
    },
}

pub fn handle_cli(cli_action: CliAction) -> Result<(), Box<dyn std::error::Error>> {
    match cli_action {
        CliAction::ImportDatabase {
            database_type: DatabaseType::Native,
            path,
        } => import_native_database::run(path),
        CliAction::ImportDatabase {
            database_type: DatabaseType::Nointro,
            path,
        } => import_nointro_database::run(path),
        CliAction::Run {
            rom,
            force_game_system: force_system,
        } => {
            if force_system.is_some() {
                tracing::warn!(
                    "Forcing a system is not recommended as it can cause mysterious problems"
                );
            }

            // run_rom::run(rom, force_system, global_config)

            Ok(())
        }
        CliAction::RunExternal {
            rom,
            force_game_system,
        } => {
            if force_game_system.is_some() {
                tracing::warn!(
                    "Forcing a system is not recommended as it can cause mysterious problems"
                );
            }

            run_external_rom::run(rom, force_game_system)
        }
        CliAction::ImportRomManually {
            path,
            game_system: system,
            name,
            symlink,
        } => import_rom_manually::run(path, system, name, symlink),
        CliAction::ImportKnownRoms {
            paths: path,
            symlink,
        } => import_known_roms::run(path, symlink),
        CliAction::VerifyRoms {
            unknown_discard,
            incorrect_discard,
        } => todo!(),
    }
}
