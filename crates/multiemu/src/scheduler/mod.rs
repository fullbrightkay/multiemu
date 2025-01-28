use crate::component::ComponentId;
use crate::machine::component_store::ComponentStore;
use itertools::Itertools;
use num::ToPrimitive;
use num::{integer::lcm, rational::Ratio, Integer};
use rangemap::RangeMap;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Serialize, Deserialize, Clone)]
pub struct Scheduler {
    current_tick: u64,
    rollover_tick: u64,
    tick_real_time: Ratio<u64>,
    // Stores precomputed periods for each component
    schedule: RangeMap<u64, Vec<ComponentId>>,
    allotted_time: Duration,
}

impl Scheduler {
    pub fn new(components: &ComponentStore) -> Self {
        let component_infos: HashMap<_, _> = components
            .iter()
            .filter_map(|(component_id, table)| {
                if let Some(schedulable_component) = &table.as_schedulable {
                    return Some((component_id, schedulable_component.timings));
                }

                None
            })
            .collect();

        for (component, component_timings) in component_infos.iter() {
            tracing::debug!(
                "Component {:?} will run {} times per second",
                component,
                component_timings
            );
        }

        let common_denominator = component_infos
            .values()
            .map(|ratio| *ratio.recip().denom())
            .fold(1, |acc, denom| acc.lcm(&denom));

        // Adjust numerators to the common denominator
        let adjusted_numerators: HashMap<_, _> = component_infos
            .iter()
            .map(|(component_id, ratio)| {
                let factor = common_denominator / ratio.denom();
                (*component_id, ratio.numer() * factor)
            })
            .collect();

        let common_multiple = adjusted_numerators
            .clone()
            .into_values()
            .reduce(lcm)
            .unwrap_or(1);

        let ratios: HashMap<_, _> = adjusted_numerators
            .iter()
            .map(|(component_id, numerator)| (*component_id, common_multiple / numerator))
            .collect();

        // Fill out the schedule
        let mut schedule = RangeMap::default();

        let mut current_tick = 0;
        while current_tick < common_denominator {
            // This is (component_id, tick_rate, run_indication)
            let to_run: Vec<_> = ratios
                .iter()
                .map(|(component_id, tick_rate)| {
                    (*component_id, current_tick % *tick_rate, *tick_rate)
                })
                .sorted_by_key(|(_, run_indication, _)| *run_indication)
                .collect();

            if to_run.len() == 1 {
                let (component_id, _, tick_rate) = to_run[0];
                let time_slice = tick_rate;
                schedule.insert(current_tick..current_tick + time_slice, vec![component_id]);
                current_tick += time_slice;
                continue;
            }

            // do the different scenarios for how many should run this turn
            match to_run
                .iter()
                .filter(|(_, run_indication, _)| *run_indication == 0)
                .count()
            {
                // Nothing is set to run here
                0 => {
                    current_tick += 1;
                }
                // Full efficient batching
                1 => {
                    let batch_size = to_run[1].2 - to_run[1].1;
                    let (component_id, _, tick_rate) = to_run[0];
                    let normalized_batch_size = batch_size / tick_rate;
                    schedule.insert(
                        current_tick..current_tick + normalized_batch_size,
                        vec![component_id],
                    );
                    current_tick += batch_size;
                }
                // Conflicted components
                _ => {
                    schedule.insert(
                        current_tick..current_tick + 1,
                        to_run
                            .into_iter()
                            .filter_map(|(component_id, run_indication, _)| {
                                if run_indication == 0 {
                                    return Some(component_id);
                                }

                                None
                            })
                            .collect(),
                    );

                    current_tick += 1;
                }
            }
        }

        let tick_real_time = Ratio::new(common_multiple, common_denominator).recip();

        tracing::debug!(
            "Schedule ticks take {:?} and restarts at tick {}",
            Duration::from_secs_f64(tick_real_time.to_f64().unwrap()),
            common_denominator
        );

        Self {
            current_tick: 0,
            rollover_tick: common_denominator,
            tick_real_time,
            schedule,
            allotted_time: Duration::from_millis(16),
        }
    }

    pub fn run(&mut self, components: &ComponentStore) {
        // TODO: This should actually be calculating how much time is between frames minus draw time
        let starting_tick = self.current_tick;
        let timestamp = Instant::now();

        // Ensure we don't overstep the framerate
        while self.allotted_time > timestamp.elapsed()
            // ensure we don't overstate the emulated timespace
            && (self.current_tick.wrapping_sub(starting_tick) as f32
                * self.tick_real_time.to_f32().unwrap())
                <  self.allotted_time.as_secs_f32()
        {
            if let Some((time_slice, component_ids)) =
                self.schedule.get_key_value(&self.current_tick)
            {
                // TODO: Run this through rayon once we can stop vulkan related concurrency issues
                for component_id in component_ids {
                    if let Some(component_info) = components
                        .get(*component_id)
                        .and_then(|table| table.as_schedulable.as_ref())
                    {
                        component_info
                            .component
                            .run(time_slice.clone().count() as u64);
                    } else {
                        panic!("Schedule referencing non existant component");
                    }
                }

                self.current_tick = self
                    .current_tick
                    .saturating_add(time_slice.clone().count() as u64);
            } else {
                self.current_tick = self.current_tick.saturating_add(1);
            }

            self.current_tick %= self.rollover_tick;
        }
    }

    pub fn too_slow(&mut self) {
        // Set our allotted time to lower but not lower than one tick
        self.allotted_time = self
            .allotted_time
            .saturating_sub(Duration::from_nanos(500))
            .max(Duration::from_secs_f32(
                self.tick_real_time.to_f32().unwrap(),
            ));

        tracing::trace!(
            "Alotted time for scheduler moved down to {:?}",
            self.allotted_time
        );
    }

    pub fn too_fast(&mut self) {
        // Set our allotted time higher but not higher than what one period takes
        self.allotted_time = self
            .allotted_time
            .saturating_add(Duration::from_nanos(500))
            .min(Duration::from_secs_f32(
                (self.tick_real_time * self.rollover_tick).to_f32().unwrap(),
            ));

        tracing::trace!(
            "Alotted time for scheduler moved up to {:?}",
            self.allotted_time
        );
    }
}
