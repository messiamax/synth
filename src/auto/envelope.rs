use super::{Control, ControlError, CtrlFunction};
use crate::{
    consts::SAMPLE_RATE,
    time::{TimeKeeper, TimeManager, TimeStamp},
    utils,
};
use std::{cell::RefCell, rc::Rc};

const TIME_RANGE: (f64, f64) = (0.0, 25.0);
const HALF_LIFE_RANGE: (f64, f64) = (0.01, 10.0);

#[derive(Debug)]
pub struct Envelope {
    attack: Control,
    decay: Control,
    sustain: Control,
    sus_half_life: Option<Control>,
    release: Control,
    time_manager: Rc<RefCell<TimeManager>>,
}

impl Envelope {
    pub fn new(
        attack: f64,
        decay: f64,
        sustain: f64,
        sus_half_life: Option<f64>,
        release: f64,
    ) -> Result<Self, ControlError> {
        Ok(Self {
            attack: match Control::from_val_in_range(attack, TIME_RANGE) {
                Ok(ctrl) => ctrl,
                Err(err) => return Err(err.set_origin("Envelope", "attack")),
            },
            decay: match Control::from_val_in_range(decay, TIME_RANGE) {
                Ok(ctrl) => ctrl,
                Err(err) => return Err(err.set_origin("Envelope", "decay")),
            },
            sustain: match Control::from_val_in_unit(sustain) {
                Ok(ctrl) => ctrl,
                Err(err) => return Err(err.set_origin("Envelope", "sustain")),
            },
            sus_half_life: match sus_half_life {
                Some(half_life) => match Control::from_val_in_range(half_life, HALF_LIFE_RANGE) {
                    Ok(ctrl) => Some(ctrl),
                    Err(err) => return Err(err.set_origin("Envelope", "sustain half life")),
                },
                None => None,
            },
            release: match Control::from_val_in_range(release, TIME_RANGE) {
                Ok(ctrl) => ctrl,
                Err(err) => return Err(err.set_origin("Envelope", "release")),
            },
            time_manager: Rc::new(RefCell::new(TimeManager::default())),
        })
    }

    pub fn new_decay(decay: f64) -> Result<Envelope, ControlError> {
        Self::new(0.0, decay, 0.0, None, 0.0)
    }

    pub fn new_ad(attack: f64, decay: f64) -> Result<Self, ControlError> {
        Self::new(attack, decay, 0.0, None, 0.0)
    }

    pub fn new_adsr(
        attack: f64,
        decay: f64,
        sustain: f64,
        release: f64,
    ) -> Result<Self, ControlError> {
        Self::new(attack, decay, sustain, None, release)
    }

    pub fn new_adsr_with_half_life(
        attack: f64,
        decay: f64,
        sustain: f64,
        sus_half_life: f64,
        release: f64,
    ) -> Result<Envelope, ControlError> {
        Self::new(attack, decay, sustain, Some(sus_half_life), release)
    }
}

impl Default for Envelope {
    fn default() -> Self {
        Self::new_adsr(0.1, 0.15, 0.8, 0.6).unwrap()
    }
}

impl Envelope {
    pub fn get_envelope(&self, sus_samples: usize, time: TimeStamp) -> Vec<f64> {
        let attack = utils::seconds_to_samples(self.attack.get_value(time));
        let decay = utils::seconds_to_samples(self.decay.get_value(time));
        let sustain = self.sustain.get_value(time);
        let release = utils::seconds_to_samples(self.release.get_value(time));

        let mut out = Vec::with_capacity(sus_samples + release);
        for i in 0..attack {
            out.push((i as f64) / (attack as f64))
        }
        for i in 0..decay {
            out.push((1.0 - (i as f64) / (decay as f64)) * (1.0 - sustain) + sustain)
        }
        if out.len() < sus_samples {
            if let Some(d_ctrl) = &self.sus_half_life {
                let sus_half_life_factor =
                    0.5_f64.powf(1.0 / (d_ctrl.get_value(time) * SAMPLE_RATE as f64));
                let remaining = sus_samples - out.len();
                for i in 0..remaining {
                    out.push(sustain * sus_half_life_factor.powi(i as i32));
                }
            } else {
                let mut sus = vec![sustain; sus_samples - out.len()];
                out.append(&mut sus);
            }
        }
        let last_sustain = *out.last().expect("error while calculating envelope");
        for i in 0..release {
            out.push((1.0 - (i as f64) / (release as f64)) * last_sustain)
        }
        out
    }
}

impl Envelope {
    pub fn set(&mut self, other: Envelope) {
        self.attack = other.attack;
        self.decay = other.decay;
        self.sustain = other.sustain;
        self.sus_half_life = other.sus_half_life;
        self.release = other.release;
    }
}

impl TimeKeeper for Envelope {
    fn set_time_manager(&mut self, time_manager: Rc<RefCell<crate::time::TimeManager>>) {
        self.time_manager = Rc::clone(&time_manager)
    }
}

impl CtrlFunction for Envelope {
    fn get_value(&self, _time: TimeStamp) -> f64 {
        panic!()
    }

    fn get_vec(&self, start: TimeStamp, samples: usize) -> Vec<f64> {
        let attack = utils::seconds_to_samples(self.attack.get_value(start));
        let decay = utils::seconds_to_samples(self.decay.get_value(start));
        let release = utils::seconds_to_samples(self.release.get_value(start));

        if samples > attack + decay + release {
            let mut vec = self.get_envelope(0, start);
            vec.resize(samples, 0.0);
            vec
        } else {
            self.get_envelope(samples - release, start)
        }
    }
}

#[cfg(test)]
mod test {}
