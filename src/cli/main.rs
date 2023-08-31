use std::io::{stdout, Write};

use anyhow::Result;
use batmon::{μWh, Measurement};
use chrono::Duration;

const DELTA_T: std::time::Duration = std::time::Duration::from_millis(500);
const N_SAMPLES: u16 = 120;

const BAT_PATH: &str = "/sys/class/power_supply/BAT0/energy_now";

pub fn main() -> Result<()> {
  let mut m: Measurement<'_, μWh> =
    Measurement::new(Duration::from_std(DELTA_T)?, N_SAMPLES, &BAT_PATH);
  m.measure(|m| {
    #[cfg(feature = "debug_println")]
    dbg!(m.dataset().len());

    let Some(sum) = std::iter::zip(m.dataset().iter(), m.dataset().iter().skip(1))
      .map(|(before, after)| after.power - before.power)
      .reduce(std::ops::Add::add) // TODO look into impl Sum for mWh etc and using itertools::sum()
      else {
        return Ok(());
      };

    #[cfg(feature = "debug_println")]
    eprintln!("dbg after return");

    let count = m.dataset().len();
    let elapsed_secs = (count * DELTA_T.as_millis() as usize) as f32 / 1000.0;
    let avg = sum / elapsed_secs;

    // clear
    print!("\r{clr}", clr = " ".repeat(120));
    print!(
      "\r{avg:.2} mWh ({count:count_pad$} Samples every {DELTA_T:?}, last sample: {last:?})",
      count_pad = (N_SAMPLES as f32).log(10.0).ceil() as usize,
      last = m.dataset().front(),
    );
    stdout().flush()?; // or else the buffer appearently doesn't get flushed for a loooong time (20secs +)
    Ok(())
  })
}
