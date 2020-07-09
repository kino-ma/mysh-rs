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
    RedirectOW(Option<&'a str>),
    RedirectAdd(Option<&'a str>),
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
                Some(">")   => output = Output::RedirectOW(token_iter.next()),
                Some(">>")  => output = Output::RedirectAdd(token_iter.next()),
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

    pub fn set_input(&mut self, dest: Input<'b>) {
        let arg_input = match self {
            Command::Normal(args) => &mut args.input,
            Command::Piped(args, _) => &mut args.input,
        };

        *arg_input = dest;
    }

    pub fn set_output(&mut self, dest: Output<'b>) {
        let arg_output = match self {
            Command::Normal(args) => &mut args.output,
            Command::Piped(args, _) => &mut args.output,
        };

        *arg_output = dest;
    }

    pub fn exec(self) -> io::Result<process::Child> {
        match self {
            Command::Normal(args) => args.exec_normal(),
            Command::Piped(args, command) => args.exec_piped(*command),
        }
    }

}


impl Args<'_> {
    fn exec_normal(self) -> io::Result<process::Child> {
        let mut p = process::Command::new(self.cmd);

        if let Input::Pipe(pipe) = self.input {
            p.stdin(pipe);
        }

        if let Output::Pipe(pipe) = self.output {
            p.stdout(pipe);
        }

        p.args(&self.args).spawn()
    }

    fn exec_piped(mut self, mut cmd2: Command) -> io::Result<process::Child> {
        self.output = Output::Pipe(process::Stdio::piped());

        let parent_out = match self.exec_normal()?.stdout {
                Some(o) => o,
                None => return Err(io::Error::new(io::ErrorKind::Other, "failed to make pipe")),
            };


        cmd2.set_input(Input::Pipe(parent_out));
        cmd2.exec()
    }
}
