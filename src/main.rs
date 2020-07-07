extern crate mysh2;

fn main() {
    use mysh2::Status;

    loop {
        match mysh2::run() {
            Ok(Status::Continue) => continue,
            Ok(Status::Exit) => break,
            Err(error) => println!("error: {}", error),
        }
    }
}


