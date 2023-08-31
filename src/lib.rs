#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use std::{
  collections::VecDeque, fmt::Display, fs, marker::PhantomData, path::Path, thread::sleep,
};

use chrono::{DateTime, Duration, Local};
use derive_more::{Add, Display, Div, From, Mul, Sub};

use anyhow::{Context, Result};

pub type NumSamples = u16;

#[allow(non_camel_case_types)]
#[derive(Debug, Display, Clone, Copy, PartialEq, PartialOrd, From, Add, Sub, Mul, Div)]
pub struct mWh(pub f32);

#[allow(non_camel_case_types)]
#[derive(Debug, Display, Clone, Copy, PartialEq, PartialOrd, From, Add, Sub, Mul, Div)]
pub struct μWh(pub f32);

impl From<μWh> for mWh {
  fn from(value: μWh) -> Self {
    Self(value.0 / 1000.0)
  }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Datapoint {
  pub power: mWh,
  pub time_delta: Duration,
} //

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
      start_time: Default::default(),
      interval,
      num_samples,
      dataset: Default::default(),
      source: source.as_ref(),
      source_unit: Default::default(),
    }
  }

  pub fn measure(&mut self, mut action: impl FnMut(&Self) -> Result<()>) -> Result<()> {
    self.start_time = Local::now();
    self.dataset.reserve_exact(self.num_samples.into());

    let mut last: Duration;

    loop {
      last = Local::now() - self.start_time;
      let dp = self._read_datapoint()?;

      // update dataset
      self.dataset.truncate((self.num_samples - 1).into()); // so the next push gets us at numSamples
      self.dataset.push_front(dp); // truncate cuts at the end, so we push_front

      // call hook
      action(self)?;

      // sleep till next measure
      sleep(self._how_long_to_sleep(last).to_std()?);
    }
    Ok(())
  }

  fn _read_datapoint(&self) -> Result<Datapoint> {
    let raw = fs::read_to_string(self.source)?;
    let numeric: u32 = str::parse(raw.trim()).with_context(|| format!("str: {raw}"))?; // microwatthours as integer, more than 4mrd would mean more than 4kWh of capacity
    let with_unit: SourceUnit = SourceUnit::from(numeric as f32);

    let dp = Datapoint {
      power: with_unit.into(),
      time_delta: Local::now() - self.start_time,
    };
    Ok(dp)
  }

  fn _how_long_to_sleep(&self, last: Duration) -> Duration {
    self.start_time + last + self.interval - Local::now()
  }

  pub fn dataset(&self) -> &VecDeque<Datapoint> {
    &self.dataset
  }
}
