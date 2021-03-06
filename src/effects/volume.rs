use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use crate::{
    control::{Control, ControlError, FunctionKeeper},
    ctrl_f::IdMap,
    time::{TimeKeeper, TimeManager, TimeStamp},
    wave::Wave,
};

use super::{EffMarker, Effect};

const VOL_RANGE: (f64, f64) = (0.0, 5.0);

#[derive(Debug)]
pub struct Volume<W> {
    phantom: PhantomData<W>,
    volume: Control,
    on: bool,
}

impl<W: Wave> Volume<W> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
            volume: Control::from_val_in_range(1.0, VOL_RANGE).unwrap(),
            on: true,
        }
    }
}

impl<W: Wave> Default for Volume<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<W: Wave> TimeKeeper for Volume<W> {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<TimeManager>>) {
        self.volume.set_time_manager(time_manager)
    }
}

impl<W: Wave> FunctionKeeper for Volume<W> {
    fn heal_sources(&mut self, id_map: &IdMap) -> Result<(), ControlError> {
        self.volume
            .heal_sources(id_map)
            .map_err(|err| err.set_origin("Volume", "volume"))
    }

    fn test_sources(&self) -> Result<(), ControlError> {
        self.volume
            .test_sources()
            .map_err(|err| err.set_origin("Volume", "volume"))
    }

    fn set_ids(&mut self) {
        self.volume.set_ids()
    }

    fn get_ids(&self) -> Vec<usize> {
        self.volume.get_ids()
    }
}

impl<W: Wave> Effect<W> for Volume<W> {
    fn apply(&self, wave: &mut W, time_triggered: TimeStamp) {
        if self.on {
            let vol = self.volume.get_vec(time_triggered, wave.len());
            wave.scale_by_vec(vol)
        }
    }

    fn set_defaults(&mut self) {
        self.volume.set_value(1.0).unwrap()
    }

    fn on(&mut self) {
        self.on = true
    }

    fn off(&mut self) {
        self.on = false
    }

    fn toggle(&mut self) {
        self.on = !self.on
    }
}

impl<W: Wave> EffMarker<W> for Volume<W> {}
