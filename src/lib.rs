pub mod token;
pub mod parse;

use std::io;
use std::error::Error;

pub enum Status {
    Continue,
    Exit,
}

pub fn run(input: &String) -> Result<Status, Box<dyn Error>> {
    if input.is_empty() {
        return Ok(Status::Exit);
    }

    let tokens = token::List::new(input);

    if let Some(parsed) = parse::Command::new(tokens) {
        exec_and_wait(parsed)?;
    }

    Ok(Status::Continue)
}

pub fn get_input() -> Result<String, io::Error> {
    let mut buffer = String::new();
    let stdin = io::stdin();

    stdin.read_line(&mut buffer)?;

    Ok(buffer.to_string())
}


fn exec_and_wait(command: parse::Command) -> Result<(), io::Error> {
    let mut child = command.exec()?;
    child.wait()?;
    Ok(())
}
