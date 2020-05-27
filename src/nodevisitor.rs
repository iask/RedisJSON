use jsonpath_lib::parser::{NodeVisitor, ParseToken};
use jsonpath_lib::Parser;
use serde::export::Formatter;
use std::fmt::Display;

pub enum StaticPathElement {
    ArrayIndex(f64),
    ObjectKey(String),
    Root,
}

impl Display for StaticPathElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use StaticPathElement::*;
        match self {
            ArrayIndex(num) => write!(f, "[{}]", num),
            ObjectKey(key) => write!(f, "[\"{}\"]", key),
            Root => write!(f, "$"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum VisitStatus {
    NotValid,
    PartialValid,
    Valid,
}

pub struct StaticPathParser {
    pub valid: VisitStatus,
    last_token: Option<ParseToken>,
    pub static_path_elements: Vec<StaticPathElement>,
}

impl StaticPathParser {
    ///
    /// Checks if path is static & valid
    ///
    pub fn check(input: &str) -> Result<Self, String> {
        let node = Parser::compile(input)?;
        let mut visitor = StaticPathParser {
            valid: VisitStatus::PartialValid,
            last_token: None,
            static_path_elements: vec![],
        };
        visitor.visit(&node);
        Ok(visitor)
    }
}

impl NodeVisitor for StaticPathParser {
    fn visit_token(&mut self, token: &ParseToken) {
        if self.valid != VisitStatus::NotValid {
            //eprintln!("visit token: {:?} -> {:?}", self.last_token, token);
            self.valid = match (&self.last_token, token) {
                (None, ParseToken::Absolute) => {
                    self.static_path_elements.push(StaticPathElement::Root);
                    VisitStatus::Valid
                }

                (Some(ParseToken::Absolute), ParseToken::In)
                | (Some(ParseToken::Absolute), ParseToken::Array)
                | (Some(ParseToken::Array), ParseToken::Key(_))
                | (Some(ParseToken::Key(_)), ParseToken::In)
                | (Some(ParseToken::Key(_)), ParseToken::Array)
                | (Some(ParseToken::ArrayEof), ParseToken::Array)
                | (Some(ParseToken::Array), ParseToken::Number(_))
                | (Some(ParseToken::ArrayEof), ParseToken::In) => VisitStatus::PartialValid,

                (Some(ParseToken::Number(num)), ParseToken::ArrayEof) => {
                    self.static_path_elements
                        .push(StaticPathElement::ArrayIndex(num.clone()));
                    VisitStatus::Valid
                }

                (Some(ParseToken::In), ParseToken::Key(key))
                | (Some(ParseToken::Key(key)), ParseToken::ArrayEof) => {
                    self.static_path_elements
                        .push(StaticPathElement::ObjectKey(key.clone()));
                    VisitStatus::Valid
                }

                _ => VisitStatus::NotValid,
            };
            self.last_token = Some(token.clone());
        }
    }
}
