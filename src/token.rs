pub struct List<'a> {
    pub tokens: Vec<&'a str>,
}

impl List<'_> {
    pub fn new<'a>(content: &'a String) -> List<'a> {
        let mut token_list = Vec::new();

        for word in content.split_whitespace() {
            token_list.push(word);
        }

        List { tokens: token_list }
    }
}
