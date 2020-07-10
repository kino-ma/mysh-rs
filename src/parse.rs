use crate::token;
use std::io;
use std::process;

pub enum Command<'a> {
    Normal(Args<'a>),
    Piped(Args<'a>, Box<Command<'a>>),
}

pub struct Args<'a> {
    cmd: &'a str,
    args: Vec<&'a str>,
    input: Input<'a>,
    output: Output<'a>,
}

pub enum Input<'a> {
    ReadFile(&'a str),
    HereDoc(&'a str),
    HereStr(&'a str),
    Pipe(process::ChildStdout),
    Stdin,
}

pub enum Output<'a> {
    RedirectOW(&'a str),
    RedirectAdd(&'a str),
    Pipe(process::Stdio),
    Stdout,
}


impl<'b> Command<'b> {
    pub fn new<'a>(list: token::List<'a>) -> Option<Command<'a>> {
        let mut token_iter = list.tokens.into_iter();

        let mut args = Vec::new();
        let     cmd    = token_iter.next()?;
        let mut input  = Input::Stdin;
        let mut output = Output::Stdout;
        let mut piped  = None;

        loop {
            match token_iter.next() {
                Some("<")   => input = Input::ReadFile(token_iter.next()?),
                Some("<<")  => input = Input::HereDoc(token_iter.next()?),
                Some("<<<") => input = Input::HereStr(token_iter.next()?),
                Some(">")   => output = Output::RedirectOW(token_iter.next()?),
                Some(">>")  => output = Output::RedirectAdd(token_iter.next()?),
                Some("|")   => {
                    let list = token_iter.collect();
                    piped = Some(Command::new(token::List{tokens:list})?);
                    break;
                }
                Some(s)     => args.push(s),
                None        => break,
            };
        }

        let args = Args {
            cmd,
            args,
            input,
            output,
        };

        match piped {
            None => Some(Command::Normal(args)),
            Some(p) => Some(Command::Piped(args, Box::new(p))),
        }

    }

    pub fn set_input(&mut self, src: Input<'b>) {
        let args = match self {
            Command::Normal(args) => args,
            Command::Piped(args, _) => args,
        };

        args.set_input(src);
    }

    pub fn set_output(&mut self, dst: Output<'b>) {
        match self {
            Command::Normal(args) => args.set_output(dst),
            Command::Piped(args, _) => args.set_output(dst),
        }
    }

    pub fn set_output_child(&mut self, dst: Output<'b>) {
        match self {
            Command::Normal(args) => args.set_output(dst),
            Command::Piped(_, cmd2) => cmd2.set_output_child(dst),
        }
    }

    pub fn exec(self) -> io::Result<process::Child> {
        match self {
            Command::Normal(args) => args.exec_normal(),
            Command::Piped(args, command) => args.exec_piped(*command),
        }
    }

}


impl<'b> Args<'b> {
    fn set_input(&mut self, src: Input<'b>) {
        self.input = src;
    }

    fn set_output(&mut self, dst: Output<'b>) {
        self.output = dst;
    }

    fn exec_normal(self) -> io::Result<process::Child> {
        use std::fs;
        use std::io::Write;

        let mut p = process::Command::new(self.cmd);
        let mut input = None;

        match self.input {
            Input::Pipe(pipe) => { p.stdin(pipe); () },
            Input::ReadFile(filename) => {
                let fd = fs::File::open(filename)?;
                p.stdin(fd);
            },
            Input::HereDoc(eof) => {
                let content = crate::get_content(eof)?;
                input = Some(content);
                p.stdin(process::Stdio::piped());
            },
            Input::HereStr(content) => {
                input = Some(content.to_string());
                p.stdin(process::Stdio::piped());
            },
            _ => (),
        }

        match self.output {
            Output::Pipe(pipe) => { p.stdout(pipe); () },
            Output::RedirectOW(filename) => {
                let fd = fs::File::create(filename)?;
                p.stdout(fd);
            },
            Output::RedirectAdd(filename) => {
                let fd = fs::OpenOptions::new()
                    .write(true)
                    .append(true)
                    .open(filename)?;
                p.stdout(fd);
            },
            _ => (),
        };

        let mut child = p.args(&self.args).spawn()?;
        if let Some(input) = input {
            let child_stdin = match child.stdin.as_mut() {
                Some(stdin) => stdin,
                None => return Err(io::Error::new(io::ErrorKind::Other, "failed to make pipe")),
            };
            child_stdin.write_all(input.as_bytes())?;
        }
        Ok(child)

    }

    fn exec_piped(mut self, mut cmd2: Command) -> io::Result<process::Child> {
        self.set_output(Output::Pipe(process::Stdio::piped()));

        let parent_out = match self.exec_normal()?.stdout {
                Some(o) => o,
                None => return Err(io::Error::new(io::ErrorKind::Other, "failed to make pipe")),
            };

        cmd2.set_input(Input::Pipe(parent_out));
        cmd2.exec()
    }
}
