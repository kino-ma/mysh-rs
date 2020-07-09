extern crate mysh2;

fn main() {
    use mysh2::*;

    loop {
        eprint!("$ ");

        let input = match get_input() {
            Ok(content) => content,
            Err(error) => {
                println!("failed to get input: {}", error);
                continue;
            }
        };

        match mysh2::run(&input) {
            Ok(Status::Continue) => continue,
            Ok(Status::Exit) => break,
            Err(error) => println!("error: {}", error),
        }
    }
}


