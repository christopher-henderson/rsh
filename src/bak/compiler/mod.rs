mod compile;
mod tokens;

use compile::*;
use std::iter::Peekable;
use shlex::Shlex;
use crate::errors::{CompilerResult, RshError};
use std::borrow::Borrow;
use std::cell::Cell;

// https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_10
struct Program {
    stmts: Option<StatementList>
}

struct StatementList {
    stmt: Box<dyn Statement>,
    stmts: Option<Box<StatementList>>
}

// ls
// ls && ls
// ls || ls
// ls && ls > lol.txt
// (ls || ls) && ls

trait Expression {}

fn compile_expression(mut src: Peekable<Shlex<'_>>) -> Box<dyn Expression> {
    let mut tokens = vec![];
    while let Some(token) = src.peek() {
        match token.as_str() {
            _ => tokens.push(src.next().unwrap())
        }
    }
    return Box::new(Unary{})
}

// fn _compile_expression<T: Expression, R: Expression>(lhs: &T, mut src: Peekable<Shlex<'_>>) -> R {
//     let mut tokens = vec![];
//     while let Some(token) = src.next() {
//         match token.as_str() {
//             tokens::AND => {
//                 // let mut and = Box::new(And{ lhs, rhs: None });
//                 // and.rhs =Some(_compile_expression(and.as_ref(), src));
//                 // return and;
//             }
//             _ => tokens.push(token)
//         }
//     }
//     return Unary{}
// }

struct Unary {}
impl Expression for Unary {}
struct And<'a> {
    lhs: &'a Box<dyn Expression>,
    rhs: Option<Box<dyn Expression>>,
}
// struct And<'a> {
//     lhs: &'a Box<dyn Expression>,
//     rhs: &'a Box<dyn Expression>
// }
impl Expression for And<'_> {}
struct Or {
    lhs: Box<dyn Expression>,
    rhs: Box<dyn Expression>
}
struct Redirect {
    lhs: Box<dyn Expression>,
    rhs: String
}
struct Append {
    lhs: Box<dyn Expression>,
    rhs: Box<dyn Expression>
}
struct Pipe {
    lhs: Box<dyn Expression>,
    rhs: Box<dyn Expression>
}


impl Compile<Peekable<Shlex<'_>>> for StatementList {
    fn compile(source: &mut Peekable<Shlex<'_>>) -> CompilerResult<Self> {
        unimplemented!()
    }
}

trait Statement {}

struct Command {
    tokens: Vec<String>
}

impl Compile<Peekable<Shlex<'_>>> for Command {
    fn compile(source: &mut Peekable<Shlex<'_>>) -> CompilerResult<Self> {
        let mut tokens = vec![];
        while let Some(token) = source.peek() {
            if tokens::reserved(token) {
                return Ok(Command{tokens})
            }
            tokens.push(source.next().unwrap())
        }
        return Ok(Command{tokens})
    }
}

impl Compile<Vec<String>> for Command {
    fn compile(source: &mut Vec<String>) -> CompilerResult<Self> where Self: std::marker::Sized {
        Ok(Command{tokens: source.clone()})
    }
}

impl Statement for Command {}

// struct Or {
//     lhs: Box<dyn Statement>,
//     rhs: StatementList
// }

#[cfg(test)]
mod tests {
    use super::*;
    use typed_arena::Arena;

    #[test]
    fn compile_command() {
        let mut src = Shlex::new("go version && go get && go build 2> lol").peekable();
        let c = Command::compile(&mut src).unwrap();
        println!("{:?}", c.tokens);
        println!("{:?}", shlex::split("go version && go get && go build 2> lol"));
    }
    
    #[test]
    fn asddd() {
        let arena = bumpalo::Bump::new();
        let u: &mut Box<dyn Expression> = arena.alloc(Box::new(Unary{}));
        // let z = arena.alloc(Box::new(And{lhs:u, rhs: unsafe{Box::new_uninit()}));
    }
    
    // #[test]
    // fn asdasdas() {
    //     let arena: Arena<Box<dyn Expression>> = typed_arena::Arena::new();
    //     let m  = arena.alloc( Box::new(Unary{}));
    //     let q = arena.alloc(Box::new(Unary{}));
    //     let asdasd: Box<dyn Expression> = Box::new(And{ lhs: Cell::new(Some(q))});
    //     let z = arena.alloc(asdasd);
    // }
}