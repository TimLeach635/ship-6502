use array2d::Array2D;
use bevy::{input::keyboard::Key, prelude::Component};

#[derive(Component)]
pub struct ShipOS {
    n_columns: usize,
    n_rows: usize,
    screen: Array2D<char>,
}

impl ShipOS {
    pub fn new(n_columns: usize, n_rows: usize) -> Self {
        let mut result = Self {
            n_columns,
            n_rows,
            screen: Array2D::filled_with(' ', n_rows, n_columns),
        };

        result.draw_box(
            Dimensions { top: 0, bottom: 10, left: 2, right: 30 },
            BoxStyle::Single,
        );
        result.draw_box(
            Dimensions { top: 5, bottom: 20, left: 20, right: 70 },
            BoxStyle::Double,
        );

        result
    }

    pub fn get_screen(&self) -> String {
        let mut result = String::new();

        for row in self.screen.rows_iter() {
            result.push_str(row.collect::<String>().as_str());
            result.push('\n');
        }

        // Remove final newline
        result.pop();

        result
    }

    pub fn handle_keyboard_input(&mut self, _key: &Key) {}

    fn draw_box(&mut self, dimensions: Dimensions, style: BoxStyle) {
        // Checks
        if dimensions.top >= dimensions.bottom {
            panic!("Top must be less than bottom (was top: {}, bottom: {})", dimensions.top, dimensions.bottom);
        }
        if dimensions.left >= dimensions.right {
            panic!("Left must be less than right (was left: {}, right: {})", dimensions.left, dimensions.right);
        }
        if dimensions.bottom >= self.n_rows {
            panic!("Bottom must be less than n_rows (was bottom: {}, n_rows: {})", dimensions.bottom, self.n_rows);
        }
        if dimensions.right >= self.n_columns {
            panic!("Right must be less than n_columns (was right: {}, n_columns: {})", dimensions.right, self.n_columns);
        }

        // Strap in for this next bit.
        // I genuinely can't think of a nicer way of doing this

        // Top left corner
        let top_left = self.screen
            .get_mut(dimensions.top, dimensions.left)
            .expect("Out of bounds");

        *top_left = match style {
            BoxStyle::Single => match top_left {
                '│' | '└' | '├' | '╞' | '╘' => '├',
                '╖' | '┐' | '┬' | '─' | '╥' => '┬',
                '┤' | '┴' | '┼' | '┘' => '┼',
                _ => '┌',
            },
            BoxStyle::Double => match top_left {
                '║' | '╟' | '╚' | '╠' | '╙' => '╠',
                '╕' | '╗' | '╦' | '═' | '╤' => '╦',
                '╣' | '╝' | '╩' | '╬' => '╬',
                _ => '╔',
            },
        };

        // Top border
        for col in (dimensions.left + 1)..dimensions.right {
            let ch = self.screen
                .get_mut(dimensions.top, col)
                .expect("Out of bounds");

            *ch = match style {
                BoxStyle::Single => match ch {
                    '│' | '┤' | '╡' | '╛' | '└' | '┴' | '├' | '┼' | '╞' | '╧' | '╘' | '╪' | '┘' => '┴',
                    '╢' | '╣' | '║' | '╝' | '╜' | '╟' | '╚' | '╩' | '╠' | '╬' | '╨' | '╙' | '╫' => '╨',
                    _ => '─',
                },
                BoxStyle::Double => match ch {
                    '│' | '┤' | '╡' | '╛' | '└' | '┴' | '├' | '┼' | '╞' | '╧' | '╘' | '╪' | '┘' => '╧',
                    '╢' | '╣' | '║' | '╝' | '╜' | '╟' | '╚' | '╩' | '╠' | '╬' | '╨' | '╙' | '╫' => '╩',
                    _ => '═',
                },
            };
        }

        // Top right corner
        let top_right = self.screen
            .get_mut(dimensions.top, dimensions.right)
            .expect("Out of bounds");

        *top_right = match style {
            BoxStyle::Single => match top_right {
                '│' | '┘' | '┤' | '╡' | '╛' => '┤',
                '╓' | '┌' | '┬' | '─' | '╥' => '┬',
                '├' | '┴' | '┼' | '└' => '┼',
                _ => '┐',
            },
            BoxStyle::Double => match top_right {
                '║' | '╢' | '╝' | '╣' | '╜' => '╣',
                '╒' | '╔' | '╦' | '═' | '╤' => '╦',
                '╠' | '╚' | '╩' | '╬' => '╬',
                _ => '╗',
            },
        };

        // Middle rows
        for row in (dimensions.top + 1)..dimensions.bottom {
            // Left border
            let left = self.screen
                .get_mut(row, dimensions.left)
                .expect("Out of bounds");

            *left = match style {
                BoxStyle::Single => match left {
                    '┤' | '╢' | '╖' | '╜' | '┐' | '┴' | '┬' | '─' | '┼' | '╨' | '╥' | '╫' | '┘' => '┤',
                    '╡' | '╕' | '╣' | '╗' | '╝' | '╛' | '╩' | '╦' | '═' | '╬' | '╧' | '╤' | '╪' => '╡',
                    _ => '│',
                },
                BoxStyle::Double => match left {
                    '┤' | '╢' | '╖' | '╜' | '┐' | '┴' | '┬' | '─' | '┼' | '╨' | '╥' | '╫' | '┘' => '╢',
                    '╡' | '╕' | '╣' | '╗' | '╝' | '╛' | '╩' | '╦' | '═' | '╬' | '╧' | '╤' | '╪' => '╣',
                    _ => '║',
                },
            };

            // Middle
            for col in (dimensions.left + 1)..dimensions.right {
                self.screen.set(row, col, ' ').expect("Out of bounds");
            }

            // Right border
            let right = self.screen
                .get_mut(row, dimensions.right)
                .expect("Out of bounds");

            *right = match style {
                BoxStyle::Single => match right {
                    '├' | '╟' | '╓' | '╙' | '┌' | '┴' | '┬' | '─' | '┼' | '╨' | '╥' | '╫' | '└' => '├',
                    '╞' | '╒' | '╠' | '╔' | '╚' | '╘' | '╩' | '╦' | '═' | '╬' | '╧' | '╤' | '╪' => '╞',
                    _ => '│',
                },
                BoxStyle::Double => match right {
                    '├' | '╟' | '╓' | '╙' | '┌' | '┴' | '┬' | '─' | '┼' | '╨' | '╥' | '╫' | '└' => '╟',
                    '╞' | '╒' | '╠' | '╔' | '╚' | '╘' | '╩' | '╦' | '═' | '╬' | '╧' | '╤' | '╪' => '╠',
                    _ => '║',
                },
            };
        }

        // Bottom left corner
        let bottom_left = self.screen
            .get_mut(dimensions.bottom, dimensions.left)
            .expect("Out of bounds");

        *bottom_left = match style {
            BoxStyle::Single => match bottom_left {
                '│' | '┌' | '├' | '╞' | '╒' => '├',
                '╜' | '┘' | '┴' | '─' | '╨' => '┴',
                '┤' | '┬' | '┼' | '┐' => '┼',
                _ => '└',
            },
            BoxStyle::Double => match bottom_left {
                '║' | '╟' | '╔' | '╠' | '╓' => '╠',
                '╛' | '╝' | '╩' | '═' | '╧' => '╩',
                '╣' | '╗' | '╦' | '╬' => '╬',
                _ => '╚',
            },
        };

        // Bottom border
        for col in (dimensions.left + 1)..dimensions.right {
            let ch = self.screen
                .get_mut(dimensions.bottom, col)
                .expect("Out of bounds");

            *ch = match style {
                BoxStyle::Single => match ch {
                    '│' | '┤' | '╡' | '╕' | '┌' | '┬' | '├' | '┼' | '╞' | '╤' | '╒' | '╪' | '┐' => '┬',
                    '╢' | '╣' | '║' | '╗' | '╖' | '╟' | '╔' | '╦' | '╠' | '╬' | '╥' | '╓' | '╫' => '╥',
                    _ => '─',
                },
                BoxStyle::Double => match ch {
                    '│' | '┤' | '╡' | '╕' | '┌' | '┬' | '├' | '┼' | '╞' | '╤' | '╒' | '╪' | '┐' => '╤',
                    '╢' | '╣' | '║' | '╗' | '╖' | '╟' | '╔' | '╦' | '╠' | '╬' | '╥' | '╓' | '╫' => '╦',
                    _ => '═',
                },
            };
        }

        // Bottom right corner
        let bottom_right = self.screen
            .get_mut(dimensions.bottom, dimensions.right)
            .expect("Out of bounds");

        *bottom_right = match style {
            BoxStyle::Single => match bottom_right {
                '│' | '┐' | '┤' | '╡' | '╕' => '┤',
                '╙' | '└' | '┴' | '─' | '╨' => '┴',
                '├' | '┬' | '┼' | '┌' => '┼',
                _ => '┘',
            },
            BoxStyle::Double => match bottom_right {
                '║' | '╢' | '╗' | '╣' | '╖' => '╣',
                '╘' | '╚' | '╩' | '═' | '╧' => '╩',
                '╠' | '╔' | '╦' | '╬' => '╬',
                _ => '╝',
            },
        };
    }
}

struct Dimensions {
    top: usize,
    bottom: usize,
    left: usize,
    right: usize,
}

enum BoxStyle {
    Single,
    Double,
}
