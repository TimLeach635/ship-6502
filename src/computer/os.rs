use std::process::Command;

pub struct OS;

impl OS {
    pub fn execute(&mut self, input: String) -> String {
        // Create iterator through parts of input
        let mut arg_iter = input.split_whitespace();

        // Extract the first part, which we take as the program name
        let prog = match arg_iter.next() {
            Some(prog) => prog,
            None => return String::new(),
        };

        // Spawn the command
        let mut command = Command::new(prog);

        // Pass all the arguments
        for arg in arg_iter {
            command.arg(arg);
        }

        // Execute command and collect output
        let output = match command.output() {
            Ok(result) => result,
            Err(_) => return "Failed to execute command".to_owned(),
        };

        // Return output
        match String::from_utf8(output.stdout) {
            Ok(result) => result,
            Err(error) => format!("{}", error),
        }
    }
}
