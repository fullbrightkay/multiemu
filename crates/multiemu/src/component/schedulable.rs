use num::rational::Ratio;
use super::Component;

pub trait SchedulableComponent: Component {
    fn run(&self, period: u32);
}
