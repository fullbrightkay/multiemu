use std::sync::Mutex;

use crate::{
    component::{schedulable::SchedulableComponent, Component, FromConfig},
    machine::ComponentBuilder,
};
use num::rational::Ratio;

pub struct Chip8Audio {
    // The CPU will set this according to what the program wants
    sound_timer: Mutex<u8>,
}

impl Chip8Audio {
    pub fn set(&self, value: u8) {
        *self.sound_timer.lock().unwrap() = value;
    }
}

impl Component for Chip8Audio {}

impl FromConfig for Chip8Audio {
    type Config = ();

    fn from_config(component_builder: &mut ComponentBuilder<Self>, _config: Self::Config) {
        component_builder
            .set_component(Self {
                sound_timer: Mutex::new(0),
            })
            .set_schedulable(Ratio::from_integer(60), [], []);
    }
}

impl SchedulableComponent for Chip8Audio {
    fn run(&self, period: u64) {
        let mut sound_timer_guard = self.sound_timer.lock().unwrap();
        *sound_timer_guard = sound_timer_guard.saturating_sub(period.try_into().unwrap_or(u8::MAX));
    }
}
