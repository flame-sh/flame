use rand::distributions::{Distribution, Uniform};
use std::error::Error;
use std::io::stdin;

fn main() -> Result<(), Box<dyn Error>> {
    let mut rng = rand::thread_rng();
    let die = Uniform::from(0.0..1.0);

    let mut input = String::new();
    stdin().read_line(&mut input)?;

    let total = input.trim().parse::<i32>()?;
    let mut sum = 0;

    for _ in 0..total {
        let x = die.sample(&mut rng) as f64;
        let y = die.sample(&mut rng) as f64;
        let dist = (x * x + y * y).sqrt();

        if dist <= 1.0 {
            sum = sum + 1;
        }
    }

    println!("{}", sum);

    Ok(())
}
