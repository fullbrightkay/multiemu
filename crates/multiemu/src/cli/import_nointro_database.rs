use crate::{
    config::GLOBAL_CONFIG,
    rom::{id::RomId, info::RomInfo, manager::RomManager, system::GameSystem},
};
use serde::Deserialize;
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use std::{fs, path::PathBuf};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Datafile {
    header: Header,
    #[serde(alias = "game")]
    machine: Vec<Machine>,
}

#[allow(dead_code)]
#[serde_as]
#[derive(Debug, Deserialize)]
struct Header {
    #[serde_as(as = "DisplayFromStr")]
    name: GameSystem,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Machine {
    #[serde(rename = "@name")]
    name: String,
    description: String,
    rom: Rom,
}

#[allow(dead_code)]
#[serde_as]
#[derive(Debug, Deserialize)]
struct Rom {
    #[serde(rename = "@name")]
    name: Option<String>,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "@sha1")]
    hash: RomId,
    status: Option<String>,
    #[serde(rename = "@url")]
    url: Option<String>,
    #[serde(rename = "@region")]
    region: Option<String>,
}

pub fn run(files: Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let global_config_guard = GLOBAL_CONFIG.try_read()?;
    let rom_manager = RomManager::new(Some(&global_config_guard.database_file))?;
    let database_transaction = rom_manager.rom_information.rw_transaction()?;

    for path in files {
        let content = fs::read_to_string(&path)?;

        // Parse XML based data file
        let data_file: Datafile = match quick_xml::de::from_str(&content) {
            Ok(file) => file,
            Err(err) => {
                tracing::error!(
                    "Failed to parse XML nointro database {}: {}",
                    path.display(),
                    err
                );
                continue;
            }
        };

        tracing::info!(
            "Found {} entries in nointro database {} for the system {}",
            data_file.machine.len(),
            path.display(),
            data_file.header.name
        );

        for entry in data_file.machine {
            database_transaction.upsert(RomInfo {
                name: Some(entry.name),
                hash: entry.rom.hash,
                system: data_file.header.name,
                region: None,
            })?;
        }
    }

    database_transaction.commit()?;

    Ok(())
}
