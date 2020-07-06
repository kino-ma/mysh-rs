extern crate mysh2;

use std::io;
use std::process;

use mysh2::token::*;

enum NextTask {
    Exec(TokenList),
    Continue,
    Exit,
}

fn main() {
    loop {
        eprint!("$ ");

        let tokens = match get_tokens() {
            NextTask::Exec(tokens) => tokens,
            NextTask::Continue => continue,
            NextTask::Exit => return,
        };

        let child = exec_and_wait(tokens);
    }
}


fn get_tokens() -> NextTask {
    let mut buffer = String::new();
    let stdin = io::stdin();

    match stdin.read_line(&mut buffer) {
        Ok(0) => return NextTask::Exit,
        Ok(_) => (),
        Err(error) => {
            println!("failed to get input: {}", error);
            return NextTask::Continue;
        }
    }

    buffer = buffer.trim().to_string();
    return NextTask::Exec(TokenList::new(&buffer));
}


fn exec_and_wait(tokens: TokenList) {
    let cmd = tokens.command();
    let args = tokens.args();

    let mut child = match process::Command::new(cmd)
        .args(args)
        .spawn() {
            Ok(child) => child,
            Err(error) => {
                eprintln!("exec {} failed: {}", cmd, error);
                return;
            }
        };

    if let Err(error) = child.wait() {
        eprintln!("wait for {} failed: {}", cmd, error);
    }
}
