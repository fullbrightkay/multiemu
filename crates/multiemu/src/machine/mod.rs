use crate::{
    component::{
        display::DisplayComponent,
        input::{EmulatedGamepadMetadata, EmulatedGamepadTypeId, InputComponent},
        memory::MemoryComponent,
        schedulable::SchedulableComponent,
        Component, ComponentId, FromConfig,
    },
    input::manager::InputManager,
    memory::{AddressSpaceId, MemoryTranslationTable},
    rom::{manager::RomManager, system::GameSystem},
    scheduler::Scheduler,
};
use component_store::ComponentStore;
use num::rational::Ratio;
use rangemap::RangeSet;
use std::{
    collections::{HashMap, HashSet},
    ops::Range,
    sync::Arc,
    time::Duration,
};

pub mod component_store;
pub mod from_system;
pub mod serialization;

#[derive(Debug)]
pub struct SchedulableComponentInfo {
    pub component: Arc<dyn SchedulableComponent>,
    pub timings: Ratio<u64>,
    pub run_after: HashSet<ComponentId>,
    pub run_before: HashSet<ComponentId>,
}

#[derive(Debug)]
pub struct DisplayComponentInfo {
    pub component: Arc<dyn DisplayComponent>,
}

#[derive(Debug)]
pub struct InputComponentInfo {
    pub component: Arc<dyn InputComponent>,
    pub registered_gamepad_types: HashMap<EmulatedGamepadTypeId, EmulatedGamepadMetadata>,
    pub registered_gamepads: Vec<EmulatedGamepadTypeId>,
}

#[derive(Debug)]
pub struct MemoryComponentInfo {
    pub component: Arc<dyn MemoryComponent>,
    pub assigned_ranges: HashMap<AddressSpaceId, RangeSet<usize>>,
}

#[derive(Debug)]
pub struct ComponentTable {
    pub component: Arc<dyn Component>,
    pub as_schedulable: Option<SchedulableComponentInfo>,
    pub as_display: Option<DisplayComponentInfo>,
    pub as_input: Option<InputComponentInfo>,
    pub as_memory: Option<MemoryComponentInfo>,
}

pub struct Machine {
    pub rom_manager: Arc<RomManager>,
    pub memory_translation_table: Arc<MemoryTranslationTable>,
    pub component_store: Arc<ComponentStore>,
    pub input_manager: Arc<InputManager>,
    pub system: GameSystem,
    pub scheduler: Scheduler,
}

impl Machine {
    pub fn build(game_system: GameSystem, rom_manager: Arc<RomManager>) -> MachineBuilder {
        MachineBuilder {
            current_component_index: ComponentId(0),
            component_store: ComponentStore::new(),
            rom_manager,
            input_manager: InputManager::default(),
            system: game_system,
            memory_translation_table: MemoryTranslationTable::default(),
        }
    }

    pub fn display_components(&self) -> impl Iterator<Item = &DisplayComponentInfo> {
        self.component_store
            .components()
            .filter_map(|table| table.as_display.as_ref())
    }

    pub fn run(&mut self) {
        self.scheduler.run(&self.component_store);
    }
}

pub struct MachineBuilder {
    memory_translation_table: MemoryTranslationTable,
    current_component_index: ComponentId,
    component_store: ComponentStore,
    input_manager: InputManager,
    pub rom_manager: Arc<RomManager>,
    pub system: GameSystem,
}

impl MachineBuilder {
    pub fn build_component<C: FromConfig>(
        mut self,
        config: C::Config,
    ) -> (MachineBuilder, ComponentId) {
        let id = self.current_component_index;
        self.current_component_index = ComponentId(
            self.current_component_index
                .0
                .checked_add(1)
                .expect("Too many components"),
        );

        let mut component_builder = ComponentBuilder {
            id,
            machine: self,
            component: None,
            as_schedulable: None,
            as_display: None,
            as_input: None,
            as_memory: None,
        };
        C::from_config(&mut component_builder, config);

        (component_builder.build(), id)
    }

    pub fn default_component<C: FromConfig>(self) -> (MachineBuilder, ComponentId)
    where
        C::Config: Default,
    {
        let config = C::Config::default();
        self.build_component::<C>(config)
    }

    pub fn insert_bus(mut self, id: AddressSpaceId, width: u8) -> MachineBuilder {
        self.memory_translation_table.insert_bus(id, width);
        self
    }

    pub fn get_component<C: Component>(&self, id: ComponentId) -> Option<Arc<C>> {
        self.component_store
            .get(id)?
            .component
            .clone()
            .into_any_arc()
            .downcast::<C>()
            .ok()
    }

    pub fn build(mut self) -> Machine {
        for (address_space_id, assigned_ranges, component_id) in self
            .component_store
            .iter()
            .filter_map(|(component_id, component_table)| {
                if let Some(memory_component_info) = &component_table.as_memory {
                    return Some((memory_component_info.assigned_ranges.iter(), component_id));
                }

                None
            })
            .flat_map(|(ranges, component_id)| {
                ranges.map(move |(address_space_id, assigned_ranges)| {
                    (address_space_id, assigned_ranges, component_id)
                })
            })
        {
            self.memory_translation_table.insert_component(
                *address_space_id,
                component_id,
                assigned_ranges.clone(),
            );
        }

        // Setup emulated gamepad types
        for (emulated_gamepad_type_id, emulated_gamepad_metadata) in self
            .component_store
            .components()
            .filter_map(|component_table| component_table.as_input.as_ref())
            .flat_map(|input_component_info| input_component_info.registered_gamepad_types.iter())
        {
            tracing::debug!(
                "Registering a gamepad {} with definition {:?}",
                emulated_gamepad_type_id,
                emulated_gamepad_metadata
            );

            self.input_manager.register_emulated_gamepad_type(
                emulated_gamepad_type_id.clone(),
                emulated_gamepad_metadata.clone(),
            );
        }

        let mut emulated_gamepad_ids: HashMap<_, Vec<_>> = HashMap::default();

        // Setup emulated gamepads
        for (raw_gamepad_id, (component_id, gamepad_type_id)) in self
            .component_store
            .iter()
            .filter_map(|(component_id, component_table)| {
                if let Some(input_component_info) = &component_table.as_input {
                    return Some((component_id, input_component_info));
                }

                None
            })
            .flat_map(|(component_id, input_component_info)| {
                input_component_info.registered_gamepads.iter().map(
                    move |gamepad_type_id: &EmulatedGamepadTypeId| (component_id, gamepad_type_id),
                )
            })
            .enumerate()
        {
            let emulated_gamepad_id = raw_gamepad_id.try_into().expect("Too many gamepads!");
            emulated_gamepad_ids
                .entry(component_id)
                .or_default()
                .push(emulated_gamepad_id);
            self.input_manager
                .register_emulated_gamepad(emulated_gamepad_id, gamepad_type_id.clone());
        }

        let component_store = Arc::new(self.component_store);

        self.memory_translation_table
            .set_component_store(component_store.clone());
        let memory_translation_table = Arc::new(self.memory_translation_table);

        let machine = Machine {
            scheduler: Scheduler::new(&component_store),
            rom_manager: self.rom_manager,
            memory_translation_table,
            component_store,
            input_manager: Arc::new(self.input_manager),
            system: self.system,
        };

        // Set the memory translation tables for everything
        for component in machine
            .component_store
            .components()
            .map(|component_table| &component_table.component)
        {
            component.set_memory_translation_table(machine.memory_translation_table.clone());
        }

        // Set up input for only input components
        for (component_id, gamepad_ids) in emulated_gamepad_ids {
            machine
                .component_store
                .get(component_id)
                .unwrap()
                .as_input
                .as_ref()
                .unwrap()
                .component
                .set_input_manager(machine.input_manager.clone(), &gamepad_ids);
        }

        machine
    }
}

pub struct ComponentBuilder<C: Component> {
    id: ComponentId,
    component: Option<Arc<C>>,
    as_schedulable: Option<SchedulableComponentInfo>,
    as_display: Option<DisplayComponentInfo>,
    as_input: Option<InputComponentInfo>,
    as_memory: Option<MemoryComponentInfo>,
    machine: MachineBuilder,
}

impl<C: Component> ComponentBuilder<C> {
    pub fn set_component(&mut self, component: C) -> &mut Self {
        let component = Arc::new(component);

        self.component = Some(component);

        self
    }

    pub fn set_schedulable(
        &mut self,
        timings: Ratio<u64>,
        run_after: impl IntoIterator<Item = ComponentId>,
        run_before: impl IntoIterator<Item = ComponentId>,
    ) -> &mut Self
    where
        C: SchedulableComponent,
    {
        self.as_schedulable = self.component.clone().map(|c| SchedulableComponentInfo {
            component: c,
            timings,
            run_after: run_after.into_iter().collect(),
            run_before: run_before.into_iter().collect(),
        });

        self
    }

    pub fn set_display(&mut self) -> &mut Self
    where
        C: DisplayComponent,
    {
        self.as_display = self
            .component
            .clone()
            .map(|c| DisplayComponentInfo { component: c });

        self
    }

    pub fn set_memory(
        &mut self,
        ranges: impl IntoIterator<Item = (AddressSpaceId, Range<usize>)>,
    ) -> &mut Self
    where
        C: MemoryComponent,
    {
        let mut assigned_ranges: HashMap<_, RangeSet<_>> = HashMap::default();

        for (address_space_id, range) in ranges.into_iter() {
            assigned_ranges
                .entry(address_space_id)
                .or_default()
                .insert(range);
        }

        self.as_memory = self.component.clone().map(|c| MemoryComponentInfo {
            component: c,
            assigned_ranges,
        });

        self
    }

    pub fn set_input(
        &mut self,
        emulated_gamepad_types: impl IntoIterator<
            Item = (EmulatedGamepadTypeId, EmulatedGamepadMetadata),
        >,
        emulated_gamepads: impl IntoIterator<Item = EmulatedGamepadTypeId>,
    ) -> &mut Self
    where
        C: InputComponent,
    {
        self.as_input = self.component.clone().map(|c| InputComponentInfo {
            component: c,
            registered_gamepad_types: emulated_gamepad_types.into_iter().collect(),
            registered_gamepads: emulated_gamepads.into_iter().collect(),
        });

        self
    }

    pub fn id(&self) -> ComponentId {
        self.id
    }

    pub fn machine(&self) -> &MachineBuilder {
        &self.machine
    }

    fn build(mut self) -> MachineBuilder {
        assert!(self.machine.component_store.0.len() == self.id.0 as usize);

        self.machine.component_store.0.push(ComponentTable {
            component: self.component.expect("Component did not initialize itself"),
            as_schedulable: self.as_schedulable,
            as_display: self.as_display,
            as_input: self.as_input,
            as_memory: self.as_memory,
        });

        self.machine
    }
}
