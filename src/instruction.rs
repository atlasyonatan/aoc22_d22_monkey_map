use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;

pub enum Instruction<N, T> {
    Step(N),
    Turn(T),
}

#[derive(PartialEq)]
pub enum Rotation {
    Clockwise,
    AntiClockwise,
}

pub fn parse_instructions<T: FromStr>(
    s: &str,
) -> Result<Vec<Instruction<T, Rotation>>, <T as FromStr>::Err> {
    lazy_static! {
        static ref REG: Regex = Regex::new(r"([RL])|(\d+)").unwrap();
    }
    REG.captures_iter(s)
        .map(|captures| {
            Ok(if let Some(mat) = captures.get(1) {
                Instruction::Turn(match mat.as_str() {
                    "L" => Rotation::Clockwise,
                    "R" => Rotation::AntiClockwise,
                    other => panic!("unknown turn symbol '{}'", other),
                })
            } else if let Some(mat) = captures.get(2) {
                Instruction::Step(mat.as_str().parse()?)
            } else {
                panic!()
            })
        })
        .collect()
}
