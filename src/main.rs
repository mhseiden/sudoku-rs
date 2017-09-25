extern crate env_logger;
extern crate sudoku;

fn main() {
    env_logger::init().unwrap();

    let solution = sudoku::solve(vec![
        (0, 2, 7),
        (0, 4, 8),
        (0, 6, 1),

        (1, 1, 9),
        (1, 3, 1),
        (1, 5, 3),
        (1, 7, 8),

        (2, 0, 5),
        (2, 4, 4),
        (2, 8, 6),

        (3, 1, 1),
        (3, 7, 4),

        (4, 0, 8),
        (4, 2, 2),
        (4, 6, 6),
        (4, 8, 7),

        (5, 1, 6),
        (5, 7, 5),

        (6, 0, 4),
        (6, 4, 6),
        (6, 8, 2),

        (7, 1, 2),
        (7, 3, 8),
        (7, 5, 4),
        (7, 7, 1),

        (8, 2, 9),
        (8, 4, 5),
        (8, 6, 4),
    ]);
    println!("{}", solution);
}