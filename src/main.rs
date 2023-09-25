use bimap::BiHashMap;
use itertools::Itertools;
use lazy_static::lazy_static;
use ndarray::Array2;
use std::{
    collections::HashMap,
    fs::File,
    hash::Hash,
    io::{BufRead, BufReader},
    ops::IndexMut,
};
pub mod array2;
pub mod instruction;
pub mod tile;
use crate::{
    array2::collect_array2,
    instruction::{parse_instructions, Instruction},
    tile::{parse_row, Tile},
};

type Board = Array2<Option<Tile>>;
type Position = (usize, usize);
const EDGE_LEN: usize = 4;
fn main() {
    let path = "../test.txt";

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
    // println!("col0: {x:?}");
    let start_col = board
        .column(start_row) //huh?
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
    for instruction in instructions.iter() {
        // println!("position: {position:?}");
        // println!("facing: {facing}");
        match instruction {
            Instruction::Step(count) => {
                position = step_wrap(&board, position, facing, *count as usize)
            }
            Instruction::Turn(rotation) => facing = sum_angles(facing as i8, *rotation) as usize,
        }
    }

    let result = 1000 * (position.1 + 1) + 4 * (position.0 + 1) + facing;

    println!("part 1: {result}");

    //part 2

    let mut blank_faces = Array2::from_shape_fn((4, 4), |(x, y)| {
        board.get((x * EDGE_LEN, y * EDGE_LEN)).is_some_and(|option| option.is_some())
    });
    println!("blank_faces:\n{blank_faces:?}");
    let mut cube_projection: CubeProjection = Array2::default((4, 4));
    let (index, blank_face) = blank_faces
        .indexed_iter_mut()
        .find(|(_, is_some)| **is_some)
        .unwrap();
    *blank_face = false;
    *cube_projection.index_mut(index) = Some((1u8, 0u8));

    let mut stack = Vec::new();
    let (col_range, row_range) = (
        (0..cube_projection.dim().0 as i32),
        (0..cube_projection.dim().1 as i32),
    );

    for (abs_angle, direction) in CARDINALS.iter().enumerate() {
        let new_index = (
            index.0 as i32 + direction.0 as i32,
            index.1 as i32 + direction.1 as i32,
        );
        if !(col_range.contains(&new_index.0) && row_range.contains(&new_index.1)) {
            continue;
        }
        let new_index = (new_index.0 as usize, new_index.1 as usize);
        if !blank_faces[new_index] {
            continue;
        }
        stack.push((index, new_index, abs_angle as i8));
    }
    // let mut push_neighbors = |index: (usize, usize)| {
    //     for (abs_angle, direction) in CARDINALS.iter().enumerate() {
    //         let new_index = (
    //             index.0 as i32 + direction.0 as i32,
    //             index.1 as i32 + direction.1 as i32,
    //         );
    //         if !(col_range.contains(&new_index.0) && row_range.contains(&new_index.1)) {
    //             continue;
    //         }
    //         let new_index = (new_index.0 as usize, new_index.1 as usize);
    //         if !blank_faces[new_index] {
    //             continue;
    //         }
    //         stack.push((index, new_index, abs_angle as i8));
    //     }
    // };
    // push_neighbors(index);

    while let Some((prev_index, index, abs_exit_angle)) = stack.pop() {
        *blank_faces.index_mut(index) = false;

        let (prev_face, orientation) = cube_projection.get(prev_index).unwrap().unwrap();
        let relative_exit_angle = sum_angles(abs_exit_angle as i8, -(orientation as i8)) as u8;
        let (face, relative_enter_angle) = CUBE_MAP.get(&(prev_face, relative_exit_angle)).unwrap();
        let abs_enter_angle = sum_angles(*relative_enter_angle as i8, orientation as i8) as u8;
        *cube_projection.index_mut(index) = Some((*face, abs_enter_angle));
        // push_neighbors(index);
        for (abs_angle, direction) in CARDINALS.iter().enumerate() {
            let new_index = (
                index.0 as i32 + direction.0 as i32,
                index.1 as i32 + direction.1 as i32,
            );
            if !(col_range.contains(&new_index.0) && row_range.contains(&new_index.1)) {
                continue;
            }
            let new_index = (new_index.0 as usize, new_index.1 as usize);
            if !blank_faces[new_index] {
                continue;
            }
            stack.push((index, new_index, abs_angle as i8));
        }
    }
    println!("projection:\n{cube_projection:?}");
    let face_map: FaceMap = cube_projection
        .indexed_iter()
        .filter_map(|(index, option)| option.and_then(|(face, angle)| Some((face, (angle, index)))))
        .collect(); //match option{
                    // Some((face, angle)) => Some((*face, (*angle, index))),
                    // None => None,
                    // ).collect();
    println!("facemap:\n{face_map:?}");
    let mut angle = RIGHT;
    let mut position = (start_col, start_row); //coincidental same as face1
                                               // println!("start position: {position:?}");
    for instruction in instructions {
        // println!("position: {position:?}");
        // println!("facing: {facing}");
        match instruction {
            Instruction::Step(count) => {
                position = step_cube(
                    &board,
                    position,
                    angle,
                    count as usize,
                    &cube_projection,
                    &face_map,
                )
            }
            Instruction::Turn(rotation) => angle = sum_angles(angle as i8, rotation) as u8,
        }
    }
    // for ((x,y), value) in cube_projection.indexed_iter_mut() {
    //     if board[(x*edge_len,y*edge_len)].is_some(){
    //         *value = Some((face_number, 0));
    //         face_number +=1;
    //     }
    // }
}

type CubeProjection = Array2<Option<(Face, Angle)>>;
type FaceMap = HashMap<Face, (Angle, (usize, usize))>;

fn sum_angles(a: i8, b: i8) -> i8 {
    (a + b).rem_euclid(MODULUS)
}

// Right = 0,
// Down = 1,
// Left = 2,
// Up = 3,
const CARDINALS: [(i8, i8); MODULUS as usize] = [(1, 0), (0, 1), (-1, 0), (0, -1)];
const MODULUS: i8 = 4;

fn step_wrap(board: &Board, mut position: Position, facing: usize, mut count: usize) -> Position {
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

fn step_cube(
    board: &Board,
    mut position: Position,
    mut angle: Angle,
    mut count: usize,
    cube_projection: &CubeProjection,
    face_map: &FaceMap,
) -> Position {
    assert_ne!(board[position], None);
    let (col_len, row_len) = board.dim();
    let (col_len, row_len) = (col_len as i32, row_len as i32);

    while count > 0 {
        let direction = CARDINALS[angle as usize];
        let step = (
            (position.0 as i32 + direction.0 as i32).rem_euclid(col_len) as usize,
            (position.1 as i32 + direction.1 as i32).rem_euclid(row_len) as usize,
        );
        match board[step] {
            Some(Tile::Solid) => {
                break;
            }
            Some(Tile::Open) => {
                position = step;
                count -= 1;
            }
            None => {
                //change angle
                let index = (position.0 / EDGE_LEN, position.1 / EDGE_LEN);
                let (face, orientation) = cube_projection.get(index).unwrap().unwrap();
                let relative_angle = sum_angles(angle as i8, -(orientation as i8)) as u8;
                let &(new_face, new_angle) = CUBE_MAP.get(&(face, relative_angle)).unwrap();
                let &(new_orientation, new_index) = face_map.get(&new_face).unwrap();
                angle = sum_angles(new_angle as i8, new_orientation as i8) as u8;

                //change position
                let edge_half = EDGE_LEN / 2;
                let relative_x = (position.0 % EDGE_LEN) as i32 - edge_half as i32;
                let relative_y = (position.1 % EDGE_LEN) as i32 - edge_half as i32;

                let new_relative_position = rotate_position((relative_x, relative_y), orientation);
                let new_x =
                    new_index.0 * EDGE_LEN + (edge_half as i32 + new_relative_position.0) as usize;
                let new_y =
                    new_index.1 * EDGE_LEN + (edge_half as i32 + new_relative_position.1) as usize;

                position = (new_x, new_y);
            }
        }
    }
    position
}

fn rotate_position(mut position: (i32, i32), angle: Angle) -> (i32, i32) {
    for _ in 0..angle {
        position = (position.1, -position.0)
    }
    position
}

type Face = u8;
type Angle = u8;

const RIGHT: u8 = 0;
const DOWN: u8 = 1;
const LEFT: u8 = 2;
const UP: u8 = 3;
lazy_static! {
    static ref CUBE_MAP: BiMap<(Face, Angle)> = BiMap(BiHashMap::from_iter([
        ((1, UP), (3, DOWN)),
        ((1, RIGHT), (2, LEFT)),
        ((1, DOWN), (4, DOWN)),
        ((1, LEFT), (5, RIGHT)),
        ((2, UP), (3, RIGHT)),
        ((2, RIGHT), (6, LEFT)),
        ((2, DOWN), (4, LEFT)),
        ((3, UP), (6, UP)),
        ((3, LEFT), (5, UP)),
        ((4, UP), (6, DOWN)),
        ((4, RIGHT), (5, DOWN)),
        ((5, LEFT), (6, RIGHT)),
    ]));
}

struct BiMap<T>(BiHashMap<T, T>);

impl<T> BiMap<T>
where
    T: Eq + PartialEq + Hash,
{
    fn get(&self, item: &T) -> Option<&T> {
        self.0
            .get_by_left(item)
            .or_else(|| self.0.get_by_right(item))
    }
}
