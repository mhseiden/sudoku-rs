#[macro_use]
extern crate log;

use std::collections::HashSet;
use std::fmt::{self, Write};

type Cell = u16;
type Index = usize;
type Guesses = HashSet<Index>;

static CELL_MASK: Cell = 0b1111111110;
static BIT_PATTERNS: [Cell; 10] = [
    0b0000000000, // 0
    0b0000000010, // 1
    0b0000000100, // 2
    0b0000001000, // 3
    0b0000010000, // 4
    0b0000100000, // 5
    0b0001000000, // 6
    0b0010000000, // 7
    0b0100000000, // 8
    0b1000000000, // 9
];

static SQUARE_I_OFFSET: [usize; 9] = [0, 3, 6, 27, 30, 33, 54, 57, 60];

#[derive(Default, Debug)]
struct Stats {
    total_guesses: usize,
    bad_guesses: usize,
}

pub fn solve<I>(i: I) -> String
where
    I: IntoIterator<Item = (Index, Index, u8)>,
{
    let mut s = Sudoku::new(i.into_iter());
    let mut guesses = Guesses::new();
    let mut stats = Stats::default();

    match try_guess(&mut stats, &mut s, &mut guesses) {
        Ok(s) => {
            info!("Stats={:?}", stats);
            s.print().unwrap()
        }
        Err(_) => panic!("internal error - no solution"),
    }
}

fn try_guess(stats: &mut Stats, s: &mut Sudoku, guesses: &mut Guesses) -> Result<Sudoku, ()> {
    let idx = find_next_guess(s, guesses);
    guesses.insert(idx);

    let cell = s.get_cell(idx);
    for (val, bp) in BIT_PATTERNS.iter().enumerate() {
        if (cell & bp) != 0 {
            debug_assert!(val > 0 && val <= 9);
            stats.total_guesses += 1;

            let mut candidate = s.clone();
            candidate.set_value(idx, val as u8);

            if candidate.reflow().is_err() {
                stats.bad_guesses += 1;
                continue;
            }

            // check if we've found a solution
            if candidate.is_solved() {
                return Ok(candidate);
            }

            // recursively make another guess
            if let Ok(solved) = try_guess(stats, &mut candidate, guesses) {
                return Ok(solved);
            }

            stats.bad_guesses += 1;
        }
    }

    guesses.remove(&idx);
    Err(())
}

// (max) TODO investigate if making a better guess improves solution times
fn find_next_guess(s: &Sudoku, guesses: &Guesses) -> Index {
    for (idx, cell) in s.cells.iter().enumerate() {
        if cell.count_ones() != 1 && !guesses.contains(&idx) {
            return idx;
        }
    }
    panic!("internal error - exhausted all guesses");
}

#[derive(Clone)]
struct Sudoku {
    cells: Box<[Cell]>,
}

impl Sudoku {
    fn new<I>(initial: I) -> Sudoku
    where
        I: Iterator<Item = (Index, Index, u8)>,
    {
        let mut s = Sudoku {
            cells: (0..81)
                .map(|_| CELL_MASK)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        };

        // 0-based coordinates and 1-based values
        for (i, j, v) in initial {
            s.set_value(cell_from_pair(i, j), v);
        }

        // initial constrain should not fail
        s.constrain().unwrap();

        s
    }

    fn print(&self) -> Result<String, fmt::Error> {
        let mut s = String::with_capacity(81 * 3);
        for i in 0..9 {
            write!(&mut s, "\n")?;
            for j in 0..9 {
                match self.get_value(cell_from_pair(i, j)) {
                    Some(v) => write!(&mut s, " {} ", v)?,
                    None => write!(&mut s, " _ ")?,
                }
            }
            write!(&mut s, "\n")?;
        }
        Ok(s)
    }

    #[allow(dead_code)]
    fn debug_print(&self) {
        for (idx, cell) in self.cells.iter().enumerate() {
            trace!("index[{:02}] : {:b}", idx, *cell);
        }
    }

    // sets a cell to hold a given bit pattern
    fn set_bit_pattern(&mut self, c: Index, bp: u16) {
        *self.get_cell_mut(c) = bp;
    }

    // sets a cell to hold a [1-9] value
    fn set_value(&mut self, c: Index, v: u8) {
        self.set_bit_pattern(c, BIT_PATTERNS[v as usize]);
    }

    // gets the solved value from a cell
    fn get_value(&self, c: Index) -> Option<u8> {
        let cell = self.get_cell(c);
        if cell.count_ones() == 1 {
            Some(cell.trailing_zeros() as u8)
        } else {
            None
        }
    }

    // gets and derefs a cell value
    fn get_cell(&self, c: Index) -> Cell {
        debug_assert!(c < 81);
        *unsafe { self.cells.get_unchecked(c) }
    }

    // gets a mutable reference to a cell value
    fn get_cell_mut(&mut self, c: Index) -> &mut Cell {
        debug_assert!(c < 81);
        unsafe { self.cells.get_unchecked_mut(c) }
    }

    // (max) XXX this assumes 'constrain' never admits wrong answers
    fn is_solved(&self) -> bool {
        self.cells.iter().all(|c| c.count_ones() == 1)
    }

    // runs constrain until no more updates can be made
    fn reflow(&mut self) -> Result<(), ()> {
        while 0 < self.constrain()? {}
        Ok(())
    }

    // applies adjacent values to a single cell
    fn constrain(&mut self) -> Result<usize, ()> {
        let mut changed = 0;

        for idx in 0..81 {
            if self.get_cell(idx).count_ones() == 1 {
                continue; // nothing to do here!
            }

            let (i, j) = pair_from_cell(idx);

            let mut row_vals = 0;
            let mut row_bp = 0b1111111110;
            for j2 in (0..9).filter(|j2| *j2 != j) {
                let cell = self.get_cell(cell_from_pair(i, j2));
                row_bp &= !cell;

                if cell.count_ones() == 1 {
                    debug_assert_eq!(0, cell & row_vals);
                    row_vals |= cell;
                }
            }

            let mut col_vals = 0;
            let mut col_bp = 0b1111111110;
            for i2 in (0..9).filter(|i2| *i2 != i) {
                let cell = self.get_cell(cell_from_pair(i2, j));
                col_bp &= !cell;

                if cell.count_ones() == 1 {
                    debug_assert_eq!(0, cell & col_vals, "duplicate @ {}", idx);
                    col_vals |= cell;
                }
            }

            let mut squ_vals = 0;
            let mut squ_bp = 0b1111111110;
            let s = square_from_cell(idx);
            for c in 0..9 {
                let idx2 = cell_from_square(s, c);
                if idx != idx2 {
                    let cell = self.get_cell(idx2);
                    squ_bp &= !cell;

                    if cell.count_ones() == 1 {
                        debug_assert_eq!(0, cell & squ_vals);
                        squ_vals |= cell;
                    }
                }
            }

            // initialize the bit pattern conflicting values
            let all_vals = row_vals | col_vals | squ_vals;
            let mut bp = !all_vals;

            // add in the non-conflicting values and apply the mask
            bp &= row_bp | col_bp | squ_bp;
            bp &= CELL_MASK;

            let cell = self.get_cell_mut(idx);
            if bp.count_ones() == 1 {
                *cell = bp;
            } else {
                let next = *cell & !all_vals;
                if *cell == next {
                    continue;
                }
                *cell = next;
            }
            changed += 1;
        }
        Ok(changed)
    }
}

fn square_from_cell(cell: Index) -> Index {
    debug_assert!(cell < 81);
    let (i, j) = pair_from_cell(cell);
    (3 * (i / 3)) + (j / 3)
}

fn pair_from_cell(cell: Index) -> (Index, Index) {
    debug_assert!(cell < 81);
    (cell / 9, cell % 9)
}

fn cell_from_pair(i: Index, j: Index) -> Index {
    debug_assert!(i < 9 && j < 9);
    (i * 9) + j
}

fn cell_from_square(s: Index, c: Index) -> Index {
    debug_assert!(s < 9 && c < 9);
    SQUARE_I_OFFSET[s] + ((9 * (c / 3)) + (c % 3))
}
