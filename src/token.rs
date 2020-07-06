pub struct TokenList {
    tokens: Vec<Token>,
}

pub struct Token {
    _kind: TokenKind,
    word: String,
}

enum TokenKind {
    Word,
    ReadFile,
    HereDoc,
    HereStr,
    RedirectOW,
    RedirectAdd,
    Pipe,
}

impl TokenList {
    pub fn new(content: &String) -> TokenList {
        let mut token_list = Vec::new();

        for word in content.split_whitespace() {
            let _kind = match word {
                "|"   => TokenKind::Pipe,
                ">"   => TokenKind::RedirectOW,
                ">>"  => TokenKind::RedirectAdd,
                "<"   => TokenKind::ReadFile,
                "<<"  => TokenKind::HereDoc,
                "<<<" => TokenKind::HereStr,
                _     => TokenKind::Word,
            };

            let token = Token { _kind, word: word.to_string() };
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
