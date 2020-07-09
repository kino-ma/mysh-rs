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

pub fn get_input() -> std::io::Result<String> {
    let mut buffer = String::new();
    let stdin = io::stdin();

    stdin.read_line(&mut buffer)?;

    Ok(buffer.to_string())
}

pub fn get_content(eof: &str) -> std::io::Result<String> {
    let mut line = String::new();
    let mut contents = Vec::new();

    loop {
        line = get_input()?;
        if line == format!("{}\n", eof) {
            break;
        }
        contents.push(line);
    }

    Ok(contents.concat())
}

fn exec_and_wait(command: parse::Command) -> Result<(), io::Error> {
    let mut child = command.exec()?;
    child.wait()?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn should_exec_normal() {
        let command = "echo hoge".to_string();
        let output = exec_and_get_output(command);
        assert_eq!(output, "hoge\n");
    }

    #[test]
    pub fn should_exec_piped() {
        let command = "echo hogehoge | sed s/hoge/fuga/".to_string();
        let output = exec_and_get_output(command);
        assert_eq!(output, "fugahoge\n");
    }

    #[test]
    pub fn should_exec_multi_piped() {
        let command = "echo hogehoge | sed s/hoge/fuga/ | rev".to_string();
        let output = exec_and_get_output(command);
        assert_eq!(output, "egohaguf\n");
    }


    #[test]
    pub fn should_exec_redirect_ow() {
        use std::fs;

        let command = "echo hoge > temp/ow1.txt".to_string();
        exec_and_wait(command);

        let out_content = std::fs::read_to_string("temp/ow1.txt")
            .expect("failed to get output content");
        assert_eq!(out_content, "hoge\n");
        fs::remove_file("temp/ow1.txt").expect("failed to remove file");
    }

    #[test]
    pub fn should_exec_redirect_add() {
        use std::fs;

        let command = "echo hoge  > temp/add1.txt".to_string();
        exec_and_wait(command);
        let command = "echo fuga >> temp/add1.txt".to_string();
        exec_and_wait(command);

        let out_content = std::fs::read_to_string("temp/add1.txt")
            .expect("failed to get output content");
        assert_eq!(out_content, "hoge\nfuga\n");
        fs::remove_file("temp/add1.txt").expect("failed to remove file");
    }


    #[test]
    pub fn should_exec_read_file() {
        use std::fs;

        let precommand = "echo hogehoge > temp/read_file1.txt".to_string();
        exec_and_wait(precommand);

        let command = "sed s/hoge/fuga/ < temp/read_file1.txt".to_string();
        let output = exec_and_get_output(command);

        assert_eq!(output, "fugahoge\n");
        fs::remove_file("temp/read_file1.txt").expect("failed to remove file");
    }

    #[test]
    pub fn should_exec_here_doc() {
        use std::fs;

        let command =
"sed s/hoge/fuga/ << EOF
hogehoge
EOF"
        .to_string();
        let output = exec_and_get_output(command);

        assert_eq!(output, "fugahoge\n");
        fs::remove_file("temp/read_file1.txt").expect("failed to remove file");
    }



    fn exec(command: String) -> std::process::Child {
        let tokens = token::List::new(&command);
        let parsed = parse::Command::new(tokens).expect("failed to parse");
        parsed.exec().expect("failed to spawn child")
    }

    fn exec_and_wait(command: String) -> std::process::ExitStatus {
        exec(command).wait().expect("failed to wait on child")
    }

    fn exec_and_get_output(command: String) -> String {
        use std::process;

        let tokens = token::List::new(&command);
        let mut parsed = parse::Command::new(tokens).expect("failed to parse");
        parsed.set_output_child(parse::Output::Pipe(process::Stdio::piped()));
        let child = parsed.exec().expect("failed to spawn child");

        let child_out = child
            .wait_with_output()
            .expect("failed to wait on child");

        String::from_utf8_lossy(&child_out.stdout).to_string()
    }
}
