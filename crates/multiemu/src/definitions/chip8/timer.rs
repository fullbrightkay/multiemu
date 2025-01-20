use std::sync::Mutex;

use crate::{
    component::{schedulable::SchedulableComponent, Component, FromConfig},
    machine::ComponentBuilder,
};
use num::rational::Ratio;

#[derive(Debug)]
pub struct Chip8Timer {
    // The CPU will set this according to what the program wants
    delay_timer: Mutex<u8>,
}

impl Chip8Timer {
    pub fn set(&self, value: u8) {
        *self.delay_timer.lock().unwrap() = value;
    }

    pub fn get(&self) -> u8 {
        *self.delay_timer.lock().unwrap()
    }
}

impl Component for Chip8Timer {}

impl FromConfig for Chip8Timer {
    type Config = ();

    fn from_config(component_builder: &mut ComponentBuilder<Self>, _config: Self::Config) {
        component_builder
            .set_component(Self {
                delay_timer: Mutex::new(0),
            })
            .set_schedulable(Ratio::from_integer(60), [], []);
    }
}

impl SchedulableComponent for Chip8Timer {
    fn run(&self, period: u32) {
        let mut delay_timer_guard = self.delay_timer.lock().unwrap();

        *delay_timer_guard = delay_timer_guard.saturating_sub(period.try_into().unwrap_or(u8::MAX));
    }
}
