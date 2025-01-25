use clap::{Parser, Subcommand, ValueEnum};
use database::{
    native::{database_native_import, NativeAction},
    nointro::{database_nointro_import, NoIntroAction},
    DatabaseAction,
};
use rom::{import::rom_import, run::rom_run, RomAction};
use std::error::Error;

pub mod database;
pub mod rom;

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
    #[command(about = Some("Commands relating to database manipulation"))]
    Database {
        #[clap(subcommand)]
        action: DatabaseAction,
    },
    #[command(about = Some("Commands relating to rom manipulation"))]
    Rom {
        #[clap(subcommand)]
        action: RomAction,
    },
}

pub fn handle_cli(cli_action: CliAction) -> Result<(), Box<dyn Error>> {
    match cli_action {
        CliAction::Database { action } => match action {
            DatabaseAction::NoIntro { action } => match action {
                NoIntroAction::Import { paths } => {
                    database_nointro_import(paths)?;
                }
            },
            DatabaseAction::Native { action } => match action {
                NativeAction::Import { paths } => {
                    database_native_import(paths)?;
                }
            },
            DatabaseAction::ScreenScraper {} => todo!(),
        },
        CliAction::Rom { action } => match action {
            RomAction::Import { symlink, paths } => {
                rom_import(paths, symlink)?;
            }
            RomAction::Run {
                roms,
                forced_system,
            } => {
                rom_run(roms, forced_system)?;
            }
        },
    }

    Ok(())
}
