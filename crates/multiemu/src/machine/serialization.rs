use super::Machine;
use crate::{component::ComponentId, scheduler::Scheduler};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, path::Path};

#[derive(Serialize, Deserialize)]
pub struct MachineState {
    pub scheduler: Scheduler,
    pub components: HashMap<ComponentId, rmpv::Value>,
}

// TODO: Replace this with a system that does less copying and supports versioning
// TODO: Replace this with a system that uses a stable id system, component ids are not stable

impl Machine {
    pub fn save_snapshot(&self, path: impl AsRef<Path>) {
        let mut file = File::create(path).unwrap();

        rmp_serde::encode::write_named(
            &mut file,
            &MachineState {
                scheduler: self.scheduler.clone(),
                components: self
                    .component_store
                    .iter()
                    .map(|(component_id, table)| (component_id, table.component.save_snapshot()))
                    .collect(),
            },
        )
        .unwrap();
    }

    pub fn load_snapshot(&mut self, path: impl AsRef<Path>) {
        let mut file = File::create(path).unwrap();
        let state: MachineState = rmp_serde::decode::from_read(&mut file).unwrap();

        self.scheduler = state.scheduler;

        for (component_id, component_state) in state.components {
            self.component_store
                .get(component_id)
                .expect("Missing component from manifest!")
                .component
                .load_snapshot(component_state);
        }
    }
}
