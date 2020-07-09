use crate::token;

pub enum Command<'a> {
    Normal(Args<'a>),
    Piped(Box<Command<'a>>, Box<Command<'a>>),
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
    Stdin,
}

pub enum Output<'a> {
    RedirectOW(Option<&'a str>),
    RedirectAdd(Option<&'a str>),
    Stdout,
}

impl Command<'_> {
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

        let command = Command::Normal(args);

        match piped {
            None => Some(command),
            Some(p) => Some(Command::Piped(Box::new(command), Box::new(p))),
        }

    }
}
