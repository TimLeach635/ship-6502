use array2d::Array2D;
use bevy::{input::keyboard::Key, prelude::Component};

use super::{ibm_byte_map::*, os::OS};

#[derive(Component)]
pub struct Terminal {
    n_columns: usize,
    n_rows: usize,
    screen_bytes: Array2D<u8>,
    cursor_idx: usize,
    input_buffer: String,
    os: OS,
}

impl Terminal {
    pub fn new(n_columns: usize, n_rows: usize) -> Self {
        Self {
            n_columns,
            n_rows,
            screen_bytes: Array2D::filled_with(0x00, n_rows, n_columns),
            cursor_idx: 2,
            input_buffer: String::new(),
            os: OS,
        }
    }

    pub fn get_screen(&self) -> String {
        let mut result = String::with_capacity(self.n_columns * self.n_rows * 2);

        // Convert IBM's bytes to UTF-8 characters
        for (row_idx, row) in self.screen_bytes.rows_iter().enumerate() {
            for (col_idx, byte) in row.enumerate() {
                if row_idx == self.n_rows - 1 && col_idx == self.cursor_idx {
                    // Solid block, to indicate cursor
                    result.push('█');
                } else {
                    result.push(map_ibm_byte_to_unicode(byte.to_owned()));
                }
            }
            result.push('\n');
        }
        // Remove final newline
        result.pop();

        result
    }

    pub fn handle_keyboard_input(&mut self, key: &Key) {
        match key {
            // Enter submits input
            Key::Enter => {
                let output = self.os.execute(self.input_buffer.clone());
                self.input_buffer = String::new();
                self.shift_lines_up();
                self.cursor_idx = 0;

                for line in output.lines() {
                    let mut line_chars = line.chars();
                    'outer: loop {
                        for idx in 0..self.n_columns {
                            match line_chars.next() {
                                Some(next_char) => {
                                    self.screen_bytes.set(
                                        self.n_rows - 1,
                                        idx,
                                        map_unicode_to_ibm_byte(next_char),
                                    ).expect("Failed to set byte correctly");
                                },
                                None => {
                                    if idx > 0 {
                                        self.shift_lines_up();
                                    }
                                    break 'outer;
                                },
                            }
                        }
                        self.shift_lines_up();
                    }
                }
            }
            // Backspace deletes the last character
            Key::Backspace => {
                if self.cursor_idx != 0 {
                    let last = self
                        .screen_bytes
                        .get_mut(self.n_rows - 1, self.cursor_idx)
                        .expect("Tried to acces out-of-bounds screen byte");
                    *last = 0x00;
                    self.cursor_idx -= 1;
                }

                self.input_buffer.pop();
            }
            // Other keys produce characters
            Key::Character(input) => {
                // Ignore control/special characters
                if !input.chars().any(|c| c.is_control()) {
                    let curr = self
                        .screen_bytes
                        .get_mut(self.n_rows - 1, self.cursor_idx)
                        .expect("Tried to access out-of-bounds screen byte");

                    let in_char = input.chars().nth(0).expect("Error getting char from input");

                    *curr = map_unicode_to_ibm_byte(in_char);
                    if self.cursor_idx < self.n_columns - 1 {
                        self.cursor_idx += 1;
                    } else {
                        self.shift_lines_up();
                        self.cursor_idx = 0;
                    }

                    self.input_buffer.push(in_char);
                }
            }
            // Spacebar seems to be a special case
            Key::Space => {
                let curr = self
                    .screen_bytes
                    .get_mut(self.n_rows - 1, self.cursor_idx)
                    .expect("Tried to acces out-of-bounds screen byte");
                *curr = 0x20;
                if self.cursor_idx < self.n_columns - 1 {
                    self.cursor_idx += 1;
                } else {
                    self.shift_lines_up();
                    self.cursor_idx = 0;
                }

                self.input_buffer.push(' ');
            }
            _ => {}
        }
    }

    fn shift_lines_up(&mut self) {
        let rows = self.screen_bytes.as_rows();
        let mut rows_without_first_line: Vec<Vec<u8>> = rows.into_iter().skip(1).collect();
        rows_without_first_line.push(vec![0x00; self.n_columns]);
        self.screen_bytes =
            Array2D::from_rows(&rows_without_first_line).expect("Error re-collecting bytes");
        // Dilemma: should it be this function's job to make the cursor go back to the beginning,
        // or should it be the calling function's job?
        // Oh god, this is the LF vs CRLF thing all over again
        // I've decided it's the calling function's job, since sometimes I want
        // to reset it to different places
    }
}
