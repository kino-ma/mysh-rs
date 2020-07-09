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
        let mut arg_input = match self {
            Command::Normal(args) => &mut args.input,
            Command::Piped(args, _) => &mut args.input,
        };

        *arg_input = dest;
    }

    pub fn exec(self) -> io::Result<()> {
        match self {
            Command::Normal(args) => args.exec_normal()?,
            Command::Piped(args, command) => args.exec_piped(*command)?,
        };

        Ok(())
    }

}


impl Args<'_> {
    fn exec_normal(self) -> io::Result<()> {
        if let Input::Pipe(p) = self.input {
            process::Command::new(self.cmd)
                .args(&self.args)
                .stdin(p)
                .spawn()
        } else {
            process::Command::new(self.cmd)
                .args(&self.args)
                .spawn()
        }?.wait()?;

        Ok(())
    }

    fn exec_piped(self, mut cmd2: Command) -> io::Result<()> {
        let parent_out = match process::Command::new(self.cmd)
            .args(&self.args)
            .stdout(process::Stdio::piped())
            .spawn()?
            .stdout
            {
                Some(o) => o,
                None => return Err(io::Error::new(io::ErrorKind::Other, "failed to make pipe")),
            };


        cmd2.set_input(Input::Pipe(parent_out));
        cmd2.exec()
    }
}
