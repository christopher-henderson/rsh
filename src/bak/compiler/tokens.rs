use std::collections::HashSet;
use std::iter::FromIterator;


pub const AND: &str = "&&";
pub const OR: &str = "||";
pub const PIPE: &str = "|";
pub const REDIRECT: &str = ">";
pub const APPEND: &str = ">>";
pub const SEMI: &str = ";";

lazy_static!(
    static ref TOKENS: HashSet<&'static str> = HashSet::from_iter(vec![
        AND, OR, PIPE, REDIRECT, APPEND, SEMI
    ]);
);

pub fn reserved(token: &str) -> bool {
    TOKENS.contains(token)
}