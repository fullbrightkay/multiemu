use crate::{
    component::{
        display::DisplayComponent,
        input::{EmulatedGamepadMetadata, InputComponent},
        memory::MemoryComponent,
        schedulable::SchedulableComponent,
        Component, ComponentId, FromConfig,
    },
    input::{manager::InputManager, GamepadPort},
    memory::MemoryTranslationTable,
    rom::manager::RomManager,
    runtime::rendering_backend::DisplayComponentFramebuffer,
    scheduler::Scheduler,
};
use downcast_rs::DowncastSync;
use num::rational::Ratio;
use rangemap::{RangeMap, RangeSet};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    ops::Range,
    sync::{Arc, Mutex},
};

pub mod from_system;

pub struct SchedulableComponentInfo {
    pub component: Arc<dyn SchedulableComponent>,
    pub timings: Ratio<u64>,
    pub run_after: HashSet<ComponentId>,
    pub run_before: HashSet<ComponentId>,
}

pub struct DisplayComponentInfo {
    pub component: Arc<dyn DisplayComponent>,
}

pub struct InputComponentInfo {
    pub component: Arc<dyn InputComponent>,
    pub gamepad_ports: Vec<Cow<'static, EmulatedGamepadMetadata>>,
}

pub struct MemoryComponentInfo {
    pub component: Arc<dyn MemoryComponent>,
    pub assigned_ranges: RangeSet<usize>,
}

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
    pub components: HashMap<ComponentId, ComponentTable>,
    pub input_manager: Arc<InputManager>,
    scheduler: Scheduler,
}

impl Machine {
    pub fn build(rom_manager: Arc<RomManager>) -> MachineBuilder {
        MachineBuilder {
            current_component_index: ComponentId(0),
            components: HashMap::default(),
            rom_manager,
            input_manager: Arc::default(),
        }
    }

    pub fn display_components(&self) -> impl Iterator<Item = &DisplayComponentInfo> {
        self.components
            .values()
            .filter_map(|table| table.as_display.as_ref())
    }

    pub fn run(&mut self) {
        self.scheduler.run(&self.components);
    }
}

pub struct MachineBuilder {
    current_component_index: ComponentId,
    components: HashMap<ComponentId, ComponentTable>,
    pub rom_manager: Arc<RomManager>,
    pub input_manager: Arc<InputManager>,
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

    pub fn get_component<C: Component>(&self, id: ComponentId) -> Option<Arc<C>> {
        self.components
            .get(&id)?
            .component
            .clone()
            .into_any_arc()
            .downcast::<C>()
            .ok()
    }

    pub fn build(self) -> Machine {
        let memory_translation_table = Arc::new(MemoryTranslationTable::new(
            // Extract mappings
            self.components
                .iter()
                .filter_map(|(component_id, component_table)| {
                    if let Some(memory_component_info) = &component_table.as_memory {
                        return Some((memory_component_info.assigned_ranges.iter(), *component_id));
                    }

                    None
                })
                .flat_map(|(ranges, component_id)| {
                    ranges.map(move |range| (range.clone(), component_id))
                })
                .collect(),
            // Extract memory components
            self.components
                .iter()
                .filter_map(|(component_id, component_table)| {
                    if let Some(memory_component_info) = &component_table.as_memory {
                        return Some((*component_id, memory_component_info.component.clone()));
                    }

                    None
                })
                .collect(),
        ));

        let machine = Machine {
            scheduler: Scheduler::new(&self.components),
            rom_manager: self.rom_manager,
            memory_translation_table,
            components: self.components,
            input_manager: self.input_manager,
        };

        // Set the memory translation tables for everything
        for component in machine
            .components
            .values()
            .map(|component_table| &component_table.component)
        {
            component.set_memory_translation_table(machine.memory_translation_table.clone());
        }

        // Set up input

        let mut gamepad_port_mappings: HashMap<_, Vec<_>> = HashMap::new();

        for (component_id, gamepad_port, metadata) in machine
            .components
            .iter()
            // Pull out the input component infomation
            .filter_map(|(component_id, component_table)| {
                component_table
                    .as_input
                    .as_ref()
                    .map(|input_component_info| (component_id, input_component_info))
            })
            // Pull out by every port
            .flat_map(|(component_id, input_component_info)| {
                input_component_info
                    .gamepad_ports
                    .iter()
                    .map(move |metadata| (component_id, metadata))
            })
            .enumerate()
            .map(|(gamepad_port_raw, (component_id, metadata))| {
                (
                    component_id,
                    GamepadPort::try_from(gamepad_port_raw).expect("Too many registered inputa"),
                    metadata,
                )
            })
        {
            machine
                .input_manager
                .register_gamepad_port(gamepad_port, metadata.clone());
            gamepad_port_mappings
                .entry(*component_id)
                .or_default()
                .push(gamepad_port);
        }

        for (component_id, gamepad_ports) in gamepad_port_mappings {
            machine
                .components
                .get(&component_id)
                .unwrap()
                .as_input
                .as_ref()
                .unwrap()
                .component
                .set_input_manager(machine.input_manager.clone(), &gamepad_ports);
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

    pub fn set_memory(&mut self, ranges: impl IntoIterator<Item = Range<usize>>) -> &mut Self
    where
        C: MemoryComponent,
    {
        self.as_memory = self.component.clone().map(|c| MemoryComponentInfo {
            component: c,
            assigned_ranges: ranges.into_iter().collect(),
        });

        self
    }

    pub fn set_input(
        &mut self,
        gamepad_ports: impl IntoIterator<Item = Cow<'static, EmulatedGamepadMetadata>>,
    ) -> &mut Self
    where
        C: InputComponent,
    {
        self.as_input = self.component.clone().map(|c| InputComponentInfo {
            component: c,
            gamepad_ports: gamepad_ports.into_iter().collect(),
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
        self.machine.components.insert(
            self.id,
            ComponentTable {
                component: self.component.expect("Component did not initialize itself"),
                as_schedulable: self.as_schedulable,
                as_display: self.as_display,
                as_input: self.as_input,
                as_memory: self.as_memory,
            },
        );

        self.machine
    }
}
