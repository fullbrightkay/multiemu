use crate::rom::{id::RomId, manager::RomManager, system::GameSystem};
use std::sync::Arc;

pub trait Runtime {
    fn launch_gui(rom_manager: Arc<RomManager>);
    fn launch_game(
        user_specified_roms: Vec<RomId>,
        forced_game_system: Option<GameSystem>,
        rom_manager: Arc<RomManager>,
    );
}
