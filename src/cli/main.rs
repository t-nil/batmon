mod args;

use std::any::{type_name, Any};

use anyhow::Result;
use batmon::{μWh, Datapoint, Measurement};
use chrono::Duration;
use itertools::Itertools;
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
        dbg!(m.len());

        let Some(sum) = std::iter::zip(m.iter(), m.iter().skip(1))
            .map(|(before, after)| after.power - before.power)
            .reduce(std::ops::Add::add)
        // TODO look into impl Sum for mWh etc and using itertools::sum()
        else {
            return Ok(());
        };

        #[cfg(feature = "debug_println")]
        eprintln!("dbg after return");

        let count = m.len();
        let elapsed_secs = ((count - 1) * Δt.num_milliseconds() as usize) as f32 / 1000.0; // n samples mean n-1 diffs
        let avg = sum / elapsed_secs;

        let get_power = |dp: &Datapoint| dp.power.0;

        //let (min, max) = match m.iter().map(get_power).minmax()
        let last = m.front().map(get_power).unwrap_or(0f32);

        Chart::new_with_y_range(300, 100, 0.0, N as f32, last - 500f32, last + 500f32)
            .lineplot(&Shape::Continuous(Box::new(|x| {
                // this breaks if datapoints are not the same temporal distance
                m.get(N.saturating_sub(x as usize))
                    .map(get_power)
                    .unwrap_or(0.0)
            })))
            .display();
        println!("Avg: {avg:.2}");
        println!("N: {N}, Δt: {Δt}");

        println!(
            "{avg:.2} ({count:count_pad$} Samples every {Δt:?}Last sample: {last:?})",
            count_pad = (N as f32).log(10.0).ceil() as usize,
            last = m.front(),
        );
        Ok(())
    })
}

trait AdditionalIteratorFns {
    fn minmax(&self, default: Self::Item) -> (Self::Item, Self::Item)
    where
        Self::Item: Clone + PartialOrd;
    type Item;
}

/*impl<X, T: Iterator<Item = X>> AdditionalIteratorFns for T {
    type Item = X;

    #[allow(unused)]
    fn minmax(&self, default: Self::Item) -> (Self::Item, Self::Item)
    where
        Self::Item: Clone + PartialOrd,
    {
        match self.minmax(default) {
            itertools::MinMaxResult::NoElements => (default.clone(), default),
            itertools::MinMaxResult::OneElement(x) => (x.clone(), x),
            itertools::MinMaxResult::MinMax(l, r) => (l, r),
        }
    }
}*/
