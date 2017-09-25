extern crate env_logger;
extern crate sudoku;

fn main() {
    env_logger::init().unwrap();

    // This is the 17-hint sudoku, which is specially designed to foil
    // backtracking algorithms. This method only needs 227 guesses.
    //
    // https://en.wikipedia.org/wiki/Sudoku_solving_algorithms
    let solution = sudoku::solve(vec![
        (1, 5, 3),
        (1, 7, 8),
        (1, 8, 5),

        (2, 2, 1),
        (2, 4, 2),

        (3, 3, 5),
        (3, 5, 7),

        (4, 2, 4),
        (4, 6, 1),

        (5, 1, 9),

        (6, 0, 5),
        (6, 7, 7),
        (6, 8, 3),

        (7, 2, 2),
        (7, 4, 1),

        (8, 4, 4),
        (8, 8, 9),
    ]);
    println!("{}", solution);
}
