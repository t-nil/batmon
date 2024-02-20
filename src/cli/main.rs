mod args;

use anyhow::Result;
use batmon::{μWh, Measurement};
use chrono::Duration;
use textplots::{Chart, Plot, Shape};

const BAT_PATH: &str = "/sys/class/power_supply/BAT0/energy_now";

pub fn main() -> Result<()> {
    let args = args::parse();
    #[allow(non_snake_case)]
    let (Δt, N) = (
        Duration::milliseconds(
            (args.delta.parse::<f64>().expect("delta has to be f64") * 1000.0) as i64,
        ),
        args.num_samples,
    );

    let mut m: Measurement<'_, μWh> = Measurement::new(Δt, N, &BAT_PATH);
    m.measure(|m| {
        #[cfg(feature = "debug_println")]
        dbg!(m.dataset().len());

        let Some(sum) = std::iter::zip(m.dataset().iter(), m.dataset().iter().skip(1))
            .map(|(before, after)| after.power - before.power)
            .reduce(std::ops::Add::add)
        // TODO look into impl Sum for mWh etc and using itertools::sum()
        else {
            return Ok(());
        };

        #[cfg(feature = "debug_println")]
        eprintln!("dbg after return");

        let count = m.dataset().len();
        let elapsed_secs = ((count - 1) * Δt.num_milliseconds() as usize) as f32 / 1000.0; // n samples mean n-1 diffs
        let avg = sum / elapsed_secs;

        Chart::new(300, 100, 0.0, N as f32)
            .lineplot(&Shape::Continuous(Box::new(|x| {
                // this breaks if datapoints are not the same temporal distance
                m.dataset()
                    .get(N.saturating_sub(x as usize))
                    .map(|dp| dp.power.0)
                    .unwrap_or(0.0)
            })))
            .display();
        println!("Avg: {avg}");
        println!("N: {N}, Δt: {Δt}");

        // clear
        //print!("\r{clr}", clr = " ".repeat(120));
        /*print!(
          "\r{avg:.2} mWh ({count:count_pad$} Samples every {DELTA_T:?}, last sample: {last:?})",
          count_pad = (N_SAMPLES as f32).log(10.0).ceil() as usize,
          last = m.dataset().front(),
        );
        stdout().flush()?;*/ // or else the buffer appearently doesn't get flushed for a loooong time (20secs +)
        Ok(())
    })
}
