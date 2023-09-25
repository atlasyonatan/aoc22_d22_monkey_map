#[derive(PartialEq, Debug)]
pub enum Tile {
    Open,
    Solid,
}

pub fn parse_row(s: &str) -> Vec<Option<Tile>> {
    s.chars()
        .map(|c| match c {
            ' ' => None,
            '.' => Some(Tile::Open),
            '#' => Some(Tile::Solid),
            other => panic!("unknown board symbol '{}'", other),
        })
        .collect()
}
