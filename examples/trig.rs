use std::str::FromStr;

use anyhow::Context;
use sliver::Angle;

enum Unit {
    Degrees,
    Radians,
    Rotations,
}

impl FromStr for Unit {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "deg" | "degrees" | "d" | "degree" => Ok(Unit::Degrees),
            "rad" | "radians" | "r" | "radian" => Ok(Unit::Radians),
            "rot" | "rotations" | "rotation" => Ok(Unit::Rotations),
            _ => Err(anyhow::anyhow!("unit must be one of deg, rad, rot")),
        }
    }
}

fn parse_angle(input: &str) -> anyhow::Result<Angle> {
    let input = input.trim();
    let mut input = input.split_whitespace();

    let number = input.next().context("Need to provide a value")?;
    let unit = input
        .next()
        .context("Need to provide a unit: rad, deg, rot")?;

    let number: f64 = number.parse().context("failed to parse value")?;
    let unit: Unit = unit.parse().context("failed to parse unit")?;

    match unit {
        Unit::Degrees => Angle::from_degrees(number),
        Unit::Radians => Angle::from_radians(number),
        Unit::Rotations => Angle::from_rotations(number),
    }
    .context("value was some kind of invalid float")
}

fn main() -> anyhow::Result<()> {
    loop {
        let value = inquire::Text::new("Compute:")
            .with_help_message("enter a number and a unit, separated by whitespace")
            .prompt()
            .context("error from user input")?;

        let angle = parse_angle(&value).context("failed to parse angle")?;

        let sin = angle.sin();
        let cos = angle.cos();

        println!("sin: {sin}");
        println!("cos: {cos}");
    }
}
