use crate::{
    component::{
        display::DisplayComponent,
        input::{ControllerKind, InputComponent},
        memory::MemoryComponent,
        schedulable::SchedulableComponent,
        Component, ComponentId, FromConfig,
    },
    memory::MemoryTranslationTable,
    rom::manager::RomManager,
    runtime::rendering_backend::DisplayComponentFramebuffer,
};
use downcast_rs::DowncastSync;
use num::rational::Ratio;
use rangemap::{RangeMap, RangeSet};
use std::{
    collections::{HashMap, HashSet},
    ops::Range,
    sync::{Arc, Mutex},
};

pub struct SchedulableComponentInfo {
    pub component: Arc<dyn SchedulableComponent>,
    pub timings: Ratio<u32>,
    pub run_after: HashSet<ComponentId>,
    pub run_before: HashSet<ComponentId>,
}

pub struct DisplayComponentInfo {
    pub component: Arc<dyn DisplayComponent>,
}

pub struct InputComponentInfo {
    pub component: Arc<dyn InputComponent>,
    pub controller_kinds: Vec<ControllerKind>,
    pub controller_port_count: usize,
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
}

impl Machine {
    pub fn build(rom_manager: Arc<RomManager>) -> MachineBuilder {
        MachineBuilder {
            current_component_index: ComponentId(0),
            components: HashMap::default(),
            rom_manager,
        }
    }

    pub fn display_components(&self) -> impl Iterator<Item = &DisplayComponentInfo> {
        self.components
            .values()
            .filter_map(|table| table.as_display.as_ref())
    }
}

pub struct MachineBuilder {
    current_component_index: ComponentId,
    components: HashMap<ComponentId, ComponentTable>,
    rom_manager: Arc<RomManager>,
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

    pub fn rom_manager(&self) -> Arc<RomManager> {
        self.rom_manager.clone()
    }

    pub fn build(self) -> Machine {
        let memory_translation_table = Arc::new(MemoryTranslationTable::new(
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
            rom_manager: self.rom_manager,
            memory_translation_table,
            components: self.components,
        };

        // Set the memory translation tables for everything
        for component in machine
            .components
            .values()
            .map(|component_table| &component_table.component)
        {
            component.set_memory_translation_table(machine.memory_translation_table.clone());
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
        timings: Ratio<u32>,
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
        controller_kinds: impl IntoIterator<Item = ControllerKind>,
        controller_port_count: usize,
    ) -> &mut Self
    where
        C: InputComponent,
    {
        self.as_input = self.component.clone().map(|c| InputComponentInfo {
            component: c,
            controller_kinds: controller_kinds.into_iter().collect(),
            controller_port_count,
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
