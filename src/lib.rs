#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::default_trait_access)]
#![feature(div_duration)]

use std::{
    borrow::BorrowMut as _,
    collections::VecDeque,
    fmt::{Display, Write},
    fs, io,
    marker::PhantomData,
    ops::Deref,
    path::Path,
    thread::sleep,
};

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Local};
use derive_more::{Add, Display, Div, From, Mul, Sub};
use log::warn;

macro_rules! display_suffix {
    ($target:ty) => {
        impl Display for $target {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0
                    .fmt(f)
                    .and(f.write_char(' '))
                    .and(f.write_str(stringify!($target)))
            }
        }
    };
}

pub type NumSamples = usize;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, From, Add, Sub, Mul, Div)]
pub struct mWh(pub f32); // TODO NaN etc

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, From, Add, Sub, Mul, Div)]
pub struct μWh(pub f32);

impl From<μWh> for mWh {
    fn from(value: μWh) -> Self {
        Self(value.0 / 1000.0)
    }
}

display_suffix!(mWh);
display_suffix!(μWh);

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Datapoint {
    pub power: mWh,
    pub time: DateTime<Local>,
}

#[derive(Debug)]
pub struct Measurement<'a, SourceUnit> {
    pub start_time: DateTime<Local>,
    pub interval: Duration,
    pub num_samples: NumSamples,
    dataset: VecDeque<Datapoint>,
    pub source: &'a Path,
    source_unit: PhantomData<SourceUnit>,
}

impl<'a, SourceUnit> Measurement<'a, SourceUnit>
where
    SourceUnit: Into<mWh> + From<f32>,
{
    pub fn new(
        interval: Duration,
        num_samples: NumSamples,
        source: &'a impl AsRef<Path>,
    ) -> Measurement<'a, SourceUnit> {
        Measurement::<'a, SourceUnit> {
            start_time: DateTime::default(),
            interval,
            num_samples,
            dataset: VecDeque::with_capacity(num_samples),
            source: source.as_ref(),
            source_unit: Default::default(),
        }
    }

    /// Start measuring at given intervals, calling `action` to process the
    /// intermediate values.
    ///
    /// # Errors
    ///
    /// Returns the error of the callback if that errors.
    ///
    pub fn measure(&mut self, mut action: impl FnMut(&Self) -> Result<()>) -> Result<()> {
        self.start_time = Local::now();

        #[cfg(any)]
        {
            let x = io::stdout().borrow_mut();
            unsafe { libc::setvbuf(x, buffer, mode, size) }
        }

        loop {
            let dp = self.read_datapoint()?;

            // update dataset
            self.dataset.truncate(self.num_samples - 1); // so the next push gets us at numSamples
            self.dataset.push_front(dp); // truncate cuts at the end, so we push_front

            // call hook
            action(self)?;

            // sleep till next measure
            sleep(dbg!(self.how_long_to_sleep().to_std()?));
        }
    }

    fn read_datapoint(&self) -> Result<Datapoint> {
        let raw = fs::read_to_string(self.source)?;
        let numeric: u32 = str::parse(raw.trim()).with_context(|| format!("str: {raw}"))?; // microwatthours as integer, more than 4mrd would mean more than 4kWh of capacity
        #[allow(clippy::cast_precision_loss)]
        let value = numeric as f32;
        let with_unit: SourceUnit = SourceUnit::from(value);

        let dp = Datapoint {
            power: with_unit.into(),
            time: Local::now(),
        };
        Ok(dp)
    }

    fn how_long_to_sleep(&self) -> Duration {
        let interval = self
            .interval
            .to_std()
            .expect("fits into std::time::Duration");
        // hack for missing modulo
        let time_passed: std::time::Duration = (Local::now() - self.start_time)
            .to_std()
            .expect("Running longer than can fit in std::time::Duration");

        let time_factor_since_start = time_passed.div_duration_f64(interval);

        let time_until_next =
            interval.mul_f64(time_factor_since_start.ceil() - time_factor_since_start);

        Duration::from_std(time_until_next).expect("Result doesn't fit into chrono::TimeDelta")
    }

    #[must_use]
    pub fn dataset(&self) -> &VecDeque<Datapoint> {
        &self.dataset
    }
}

impl<'a, T> Deref for Measurement<'a, T> {
    type Target = VecDeque<Datapoint>;

    fn deref(&self) -> &Self::Target {
        &self.dataset
    }
}
