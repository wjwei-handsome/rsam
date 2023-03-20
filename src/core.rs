use rand::{distributions::Uniform, prelude::Distribution, Rng};

fn init_fill<I, T>(iter: &mut I, sample: &mut [T]) -> Option<()>
where
    I: Iterator<Item = T>,
{
    for element in sample.iter_mut() {
        // here will consume the iterator in len(sample) times
        *element = match iter.next() {
            Some(n) => n,
            None => return None, // return None if exhausted, actually NERVER due to check before
        };
    }

    Some(())
}

pub(crate) fn reservoir_sample<R, I, T>(mut iter: I, sample: &mut [T], rng: &mut R)
where
    R: Rng + ?Sized,
    I: Iterator<Item = T>,
{
    // Fill the sample array from the reservoir
    if let None = init_fill(&mut iter, sample) {
        return;
    }

    // Init the random index using the uniform distribution
    let random_index = Uniform::new(0, sample.len());
    // Generate the random index
    let mut w: f64 = (rng.gen::<f64>().ln() / sample.len() as f64).exp();

    loop {
        // generate the random addeed jumper
        let jumper = (rng.gen::<f64>().ln() / (1.0 - w).ln()).floor() as usize + 1;
        println!("tmp_adder: {}", jumper);

        match iter.nth(jumper - 1) {
            Some(n) => {
                // Swap the element
                sample[random_index.sample(rng)] = n;
                // Update the w
                w *= (rng.gen::<f64>().ln() / sample.len() as f64).exp();
            }
            None => {
                // If exhausted, break the loop
                break;
            }
        }
    }
}
