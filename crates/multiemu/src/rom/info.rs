use super::{id::RomId, region::RomRegion, system::GameSystem};
use native_db::native_db;
use native_db::ToKey;
use native_model::native_model;
use native_model::Model;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[native_model(id = 1, version = 1)]
#[native_db]
pub struct RomInfo {
    #[primary_key]
    pub id: RomId,
    pub name: Option<String>,
    pub system: GameSystem,
    pub region: Option<RomRegion>,
}
