use crate::Environment;
use std::iter::Peekable;
use crate::errors::{InterpreterResult, InterpreterError};

const ESCAPE: char = '\\';
const SUBSTITUTION: char = '$';
const OPEN: char = '{';
const CLOSE: char = '}';

pub fn substitution<S: AsRef<str>>(s: S, env: &Environment) -> InterpreterResult<String> {
    let mut sub = String::with_capacity(s.as_ref().len());
    let mut chars = s.as_ref().chars().peekable();
    loop {
        match chars.next() {
            Some(SUBSTITUTION) => {
                if sub.ends_with(ESCAPE) {
                    sub.pop();
                    sub.push(SUBSTITUTION);
                    continue;
                }
                match chars.peek() {
                    Some(space) if space.is_whitespace() => sub.push(SUBSTITUTION),
                    None => sub.push(SUBSTITUTION),
                    Some(&OPEN) => {
                        chars.next();
                        sub.push_str(delimited(&mut chars, env)?.as_str())
                    }
                    Some(_) => sub.push_str(longest_match(&mut chars, env).as_str()),
                }
            },
            Some(other) => sub.push(other),
            None => return Ok(sub)
        }
    }
}

fn delimited<T: Iterator<Item=char>>(stream: &mut Peekable<T>, env: &Environment) -> InterpreterResult<String> {
    let mut varname = String::new();
    loop {
        match stream.peek() {
            None => return Err(InterpreterError{message:"unclosed delimiter".to_string()}),
            Some(c) if c.is_whitespace() => return Err(InterpreterError{message:"bad substitution".to_string()}),
            Some(&CLOSE) => {
                stream.next();
                break;
            },
            Some(_) => varname.push(stream.next().unwrap())
        }
    }
    Ok(resolve(varname, env))
}

fn longest_match<T: Iterator<Item=char>>(stream: &mut Peekable<T>, env: &Environment) -> String {
    let mut varname = String::new();
    loop {
        match stream.peek() {
            None => break,
            Some(c) if c.is_whitespace() => break,
            Some(_) => varname.push(stream.next().unwrap())
        }
    }
    resolve(varname, env)
}

fn resolve(varname: String, env: &Environment) -> String {
    env.get(&varname).unwrap_or(&"".to_string()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_longest() {
        let mut env = Environment::default();
        env.insert("abcd".to_string(), "bob".to_string());
        let sub = longest_match(&mut "abcd e".chars().peekable(), &env);
        assert_eq!(sub, "bob".to_string())
    }

    #[test]
    fn test_longest_no_match() {
        let mut env = Environment::default();
        env.insert("abcd".to_string(), "b".to_string());
        let sub = longest_match(&mut "nope".chars().peekable(), &env);
        assert_eq!(sub, "".to_string())
    }

    #[test]
    fn test_longest_single() {
        let mut env = Environment::default();
        env.insert("a".to_string(), "bob".to_string());
        let sub = longest_match(&mut "a bcd".chars().peekable(), &env);
        assert_eq!(sub, "bob".to_string())
    }

    #[test]
    fn test_longest_contains_single() {
        let mut env = Environment::default();
        env.insert("a".to_string(), "bob".to_string());
        let sub = longest_match(&mut "abcd a".chars().peekable(), &env);
        assert_eq!(sub, "".to_string())
    }

    #[test]
    fn test_longest_contains_longer() {
        let mut env = Environment::default();
        env.insert("abcd".to_string(), "bob".to_string());
        env.insert("a".to_string(), "alice".to_string());
        let sub = longest_match(&mut "abcd a".chars().peekable(), &env);
        assert_eq!(sub, "bob".to_string())
    }

    #[test]
    fn test_delimiter() {
        let mut env = Environment::default();
        env.insert("abcd".to_string(), "bob".to_string());
        let sub = delimited(&mut "abcd} e".chars().peekable(), &env).unwrap();
        assert_eq!(sub, "bob".to_string())
    }

    #[test]
    fn test_delimited_no_match() {
        let mut env = Environment::default();
        env.insert("abcd".to_string(), "b".to_string());
        let sub = delimited(&mut "nope}".chars().peekable(), &env).unwrap();
        assert_eq!(sub, "".to_string())
    }

    #[test]
    fn test_delimited_single() {
        let mut env = Environment::default();
        env.insert("a".to_string(), "bob".to_string());
        let sub = delimited(&mut "a} bcd".chars().peekable(), &env).unwrap();
        assert_eq!(sub, "bob".to_string())
    }

    #[test]
    fn test_delimited_contains_single() {
        let mut env = Environment::default();
        env.insert("a".to_string(), "bob".to_string());
        let sub = delimited(&mut "abcd} a".chars().peekable(), &env).unwrap();
        assert_eq!(sub, "".to_string())
    }

    #[test]
    fn test_delimited_contains_longer() {
        let mut env = Environment::default();
        env.insert("abcd".to_string(), "bob".to_string());
        env.insert("a".to_string(), "alice".to_string());
        let sub = delimited(&mut "abcd} a".chars().peekable(), &env).unwrap();
        assert_eq!(sub, "bob".to_string())
    }
    
    #[test]
    fn test_delimited_no_close() {
        let mut env = Environment::default();
        env.insert("abcd".to_string(), "bob".to_string());
        env.insert("a".to_string(), "alice".to_string());
        let sub = delimited(&mut "abcd".chars().peekable(), &env);
        if sub.is_ok() {
            panic!(sub.unwrap())
        }
    }

    #[test]
    fn test_delimited_space() {
        let mut env = Environment::default();
        env.insert("abcd".to_string(), "bob".to_string());
        env.insert("a".to_string(), "alice".to_string());
        let sub = delimited(&mut "abc b}".chars().peekable(), &env);
        if sub.is_ok() {
            panic!(sub.unwrap())
        }
    }
    
    #[test]
    fn test_substitution() {
        let mut env = Environment::default();
        env.insert("abcd".to_string(), "bob".to_string());
        env.insert("a".to_string(), "alice".to_string());
        let got= substitution("hello ${abcd}, say hello to $a".to_string(), &env).unwrap();
        assert_eq!("hello bob, say hello to alice".to_string(), got)
    }

    #[test]
    fn test_substitution_inserted() {
        let mut env = Environment::default();
        env.insert("a".to_string(), "b".to_string());
        let got= substitution("a$a".to_string(), &env).unwrap();
        assert_eq!("ab".to_string(), got)
    }

    #[test]
    fn test_substitution_inserted_delimited() {
        let mut env = Environment::default();
        env.insert("a".to_string(), "b".to_string());
        let got= substitution("a${a}a".to_string(), &env).unwrap();
        assert_eq!("aba".to_string(), got)
    }
}