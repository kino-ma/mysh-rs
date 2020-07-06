pub struct TokenList {
    tokens: Vec<Token>,
}

pub struct Token {
    kind: TokenKind,
    word: String,
}

enum TokenKind {
    Word,
    ReadFile,
    Heredoc,
    Herestr,
    RedirectOw,
    RedirectAdd,
    Pipe,
    Exit,
}

impl TokenList {
    pub fn new(content: &String) -> TokenList {
        let mut token_list = Vec::new();

        for word in content.split(" ") {
            let kind = match word {
                "|" => TokenKind::Pipe,
                ">" => TokenKind::Pipe,
                ">>" => TokenKind::Pipe,
                _ => TokenKind::Word,
            };

            let token = Token { kind, word: word.to_string() };
            token_list.push(token);
        }

        return TokenList { tokens: token_list };
    }

    pub fn command(&self) -> &str {
        &self.tokens[0].word
    }

    pub fn args(&self) -> Vec<&std::ffi::OsStr> {
        self.tokens[1..].iter().map(|t| std::ffi::OsStr::new(&t.word)).collect()
    }
}
