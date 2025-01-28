use super::ComponentTable;
use crate::component::ComponentId;

/// [ComponentId]s are sequential so we can make this optimization
#[derive(Debug)]
pub struct ComponentStore(pub(super) Vec<ComponentTable>);

impl ComponentStore {
    pub fn new() -> Self {
        Self(Vec::default())
    }

    pub fn get(&self, component_id: ComponentId) -> Option<&ComponentTable> {
        self.0.get(component_id.0 as usize)
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (ComponentId, &'a ComponentTable)> + use<'a> {
        self.0.iter().enumerate().map(|(index, component_table)| {
            (
                ComponentId(index.try_into().expect("Too many components")),
                component_table,
            )
        })
    }

    pub fn ids<'a>(&'a self) -> impl Iterator<Item = ComponentId> + use<'a> {
        self.iter().map(|(component_id, _)| component_id)
    }

    pub fn components<'a>(&'a self) -> impl Iterator<Item = &'a ComponentTable> + use<'a> {
        self.iter().map(|(_, component_table)| component_table)
    }
}
