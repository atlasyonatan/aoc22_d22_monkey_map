use itertools::Itertools;
use ndarray::Array2;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};
pub mod array2;
pub mod instruction;
pub mod tile;
use crate::{
    array2::collect_array2,
    instruction::{parse_instructions, Rotation},
    tile::{parse_row, Tile},
};

type Board = Array2<Option<Tile>>;
type Position = (usize, usize);
fn main() {
    let path = "../input.txt";
    let file = File::open(path).unwrap();
    let groups = BufReader::new(file)
        .lines()
        .map(Result::unwrap)
        .group_by(|line| line != "");
    let mut sections = groups
        .into_iter()
        .filter_map(|(bool, group)| bool.then_some(group));
    let board_lines = sections.next().unwrap();
    let board = collect_array2(board_lines.map(|line| parse_row(&line))).unwrap();

    let start_row = 0usize;
    let x = board.column(0);
    // println!("col0: {x:?}");
    let start_col = board
        .column(start_row)//huh?
        .indexed_iter()
        .find_map(|(column, cell)| match cell {
            Some(tile::Tile::Open) => Some(column),
            _ => None,
        })
        .unwrap();
    // println!("start_col: {start_col}");
    let instructions_line = sections.next().unwrap().next().unwrap();
    let instructions = parse_instructions::<u8>(&instructions_line).unwrap();

    let mut facing = 0;
    let mut position = (start_col, start_row);
    // println!("start position: {position:?}");
    for instruction in instructions {
        // println!("position: {position:?}");
        // println!("facing: {facing}");
        match instruction {
            instruction::Instruction::Step(count) => {
                position = step(&board, position, facing, count as usize)
            }
            instruction::Instruction::Turn(rotation) => facing = turn(facing, rotation),
        }
        
        println!()
    }

    let result = 1000 * (position.1 + 1) + 4 * (position.0 + 1) + facing;

    println!("part 1: {result}");
}

// Right = 0,
// Down = 1,
// Left = 2,
// Up = 3,
const CARDINALS: [(i8, i8); MODULUS as usize] = [(1, 0), (0, 1), (-1, 0), (0, -1)];
const MODULUS: i32 = 4;
fn turn(facing: usize, rotation: Rotation) -> usize {
    (facing as i32
        + if Rotation::Clockwise == rotation {
            -1
        } else {
            1
        })
    .rem_euclid(MODULUS) as usize
}

fn step(board: &Board, mut position: Position, facing: usize, mut count: usize) -> Position {
    assert_ne!(board[position], None);
    let direction = CARDINALS[facing];
    let mut scale = 1;
    let (col_len, row_len) = board.dim();
    let (col_len, row_len) = (col_len as i32, row_len as i32);
    while count > 0 {
        let step = (
            (position.0 as i32 + scale * (direction.0 as i32)).rem_euclid(col_len) as usize,
            (position.1 as i32 + scale * (direction.1 as i32)).rem_euclid(row_len) as usize,
        );
        match board[step] {
            Some(Tile::Solid) => {
                break;
            }
            Some(Tile::Open) => {
                position = step;
                count -= 1;
                scale = 1;
            }
            None => {
                scale += 1;
            }
        }
    }
    position
}
