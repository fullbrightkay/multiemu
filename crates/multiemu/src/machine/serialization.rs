use super::Machine;
use crate::{component::ComponentId, scheduler::Scheduler};
use std::collections::HashMap;

pub struct MachineState {
    pub scheduler: Scheduler,
    pub components: HashMap<ComponentId, rmpv::Value>,
}

/// TODO: Replace this with a system that does less copying

impl Machine {
    pub fn save_snapshot(&self) -> MachineState {
        MachineState {
            scheduler: self.scheduler.clone(),
            components: self
                .components
                .iter()
                .map(|(component_id, table)| (*component_id, table.component.save_snapshot()))
                .collect(),
        }
    }

    pub fn load_snapshot(&mut self, state: MachineState) {
        self.scheduler = state.scheduler;

        for (component_id, component_state) in state.components {
            self.components
                .get(&component_id)
                .expect("Missing component from manifest!")
                .component
                .load_snapshot(component_state);
        }
    }
}
