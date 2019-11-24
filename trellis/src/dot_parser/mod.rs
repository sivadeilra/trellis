use std::io;

pub struct DotParser {}

pub struct DotLexer<'a> {
    s: &'a str,
    pos: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DotLexerError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DotKeyword {
    Strict,
    Graph,
    Digraph,
    Node,
    Edge,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DotToken<'a> {
    Id(&'a str),
    Numeral(i64),
    String(&'a str),
    HtmlString(&'a str),
    Keyword(DotKeyword),
}

impl DotLexer {
    pub fn next_token(&mut self) -> Result<DotToken, DotLexerError> {}
}
