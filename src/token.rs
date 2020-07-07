pub struct List {
    tokens: Vec<Node>,
}

pub struct Node {
    _kind: Kind,
    word: String,
}

enum Kind {
    Word,
    ReadFile,
    HereDoc,
    HereStr,
    RedirectOW,
    RedirectAdd,
    Pipe,
    Nothing,
}

impl List {
    pub fn new(content: &String) -> List {
        let mut token_list = Vec::new();

        for word in content.split_whitespace() {
            let _kind = match word {
                "|"   => Kind::Pipe,
                ">"   => Kind::RedirectOW,
                ">>"  => Kind::RedirectAdd,
                "<"   => Kind::ReadFile,
                "<<"  => Kind::HereDoc,
                "<<<" => Kind::HereStr,
                ""    => Kind::Nothing,
                _     => Kind::Word,
            };

            let node = Node { _kind, word: word.to_string() };
            token_list.push(node);
        }

        List { tokens: token_list }
    }

    pub fn command(&self) -> Option<&str> {
        if self.tokens.len() != 0 {
            Some(&self.tokens[0].word)
        } else {
            None
        }
    }

    pub fn args(&self) -> Vec<&std::ffi::OsStr> {
        self.tokens[1..].iter().map(|t| std::ffi::OsStr::new(&t.word)).collect()
    }
}
