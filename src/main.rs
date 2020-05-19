use bumpalo::Bump;
use shlex::{split, Shlex};
use std::io::{BufReader, Cursor, Seek};
use std::iter::Peekable;

struct Compiler<'a> {
    arena: bumpalo::Bump,
    stmts: Option<StatementList<'a>>,
}

impl<'a> Compiler<'a> {
    pub fn new() -> Self {
        Compiler {
            arena: Bump::new(),
            stmts: None,
        }
    }

    pub fn compile(&'a mut self, mut lexer: Lexer) -> Option<Program<'a>> {
        self.compile_statement_list(&mut lexer)
    }

    pub fn compile_statement_list(&'a self, lexer: &mut Lexer) -> Option<StatementList<'a>> {
        if lexer.peek().is_none() {
            return None;
        }
        Some(StatementList {
            stmt: self.compile_statement(lexer),
            smts: match self.compile_statement_list(lexer) {
                None => None,
                Some(stmts) => Some(Box::new(stmts)),
            }
        })
    }

    pub fn compile_statement(&'a self, lexer: &mut Lexer) -> Box<&'a dyn Statement> {
        let mut tokens = vec![];
        while let Some(token) = lexer.next() {
            match token.as_str() {
                ";" | "\n" => break,
                "&&" => return And::new(&self.arena, Command::new(&self.arena, tokens), self.compile_statement(lexer)),
                "||" => return Or::new(&self.arena, Command::new(&self.arena, tokens), self.compile_statement(lexer)),
                _ => tokens.push(token.clone()),
            }
        }
        return Box::new(self.arena.alloc(Command { tokens }));
    }
}
type Program<'a> = StatementList<'a>;

struct StatementList<'a> {
    stmt: Box<&'a dyn Statement>,
    smts: Option<Box<StatementList<'a>>>,
}

impl Statement for StatementList<'_> {
    fn eval(&self) {
        self.stmt.eval();
        if let Some(list) = &self.smts {
            list.eval();
        }
    }
}

trait Statement {
    fn eval(&self);
}

struct Command {
    tokens: Vec<String>,
}

impl Command {
    fn new(arena: & Bump, tokens: Vec<String>) -> Box<& dyn Statement> {
        return Box::new(arena.alloc(Command{tokens}))
    }
}

struct And<'a> {
    lhs: Box<&'a dyn Statement>,
    rhs: Box<&'a dyn Statement>,
}

impl <'a> And<'a> {
    fn new(arena: &'a Bump, lhs: Box<&'a dyn Statement>, rhs: Box<&'a dyn Statement>) -> Box<&'a dyn Statement> {
        return Box::new(arena.alloc(And{lhs, rhs}))
    }
}

struct Or<'a> {
    lhs: Box<&'a dyn Statement>,
    rhs: Box<&'a dyn Statement>,
}

impl <'a> Or<'a> {
    fn new(arena: &'a Bump, lhs: Box<&'a dyn Statement>, rhs: Box<&'a dyn Statement>) -> Box<&'a dyn Statement> {
        return Box::new(arena.alloc(Or{lhs, rhs}))
    }
}

impl Statement for Command {
    fn eval(&self) {
        println!("Executing {:?}", self.tokens)
    }
}

impl Statement for And<'_> {
    fn eval(&self) {
        self.lhs.eval();
        println!("AND");
        self.rhs.eval();
    }
}

impl Statement for Or<'_> {
    fn eval(&self) {
        self.lhs.eval();
        println!("OR");
        self.rhs.eval();
    }
}

const OR: &str = "||";

// pub struct Lexer<T> {
//     inner: Vec<T>,
//     pos: usize,
// }
//
// impl From<Vec<String>> for Lexer<String> {
//     fn from(tokens: Vec<String>) -> Self {
//         Lexer {
//             inner: tokens,
//             pos: 0,
//         }
//     }
// }
//
// impl<T: PartialEq> Lexer<T> {
//     pub fn peek(&self) -> Option<&T> {
//         self.inner.get(self.pos)
//     }
//
//     pub fn next(&mut self) -> Option<&T> {
//         if let Some(t) = self.inner.get(self.pos) {
//             self.pos += 1;
//             Some(t)
//         } else {
//             None
//         }
//     }
//
//     pub fn seek(&mut self, seek: i64) {
//         self.pos = (self.pos as i64 + seek) as usize;
//     }
// }

type Lexer<'a> = Peekable<Shlex<'a>>;

fn main() {
    // let arena = bumpalo::Bump::new();
    // let mut cmd = arena.alloc(Command {
    //     tokens: vec!["go".to_string(), "version".to_string()],
    // });
    // let mut rhs = arena.alloc(Command {
    //     tokens: vec!["go".to_string(), "version".to_string()],
    // });
    // let mut cmd = arena.alloc(And {
    //     lhs: Box::new(cmd),
    //     rhs: None,
    // });
    // cmd.rhs = Some(Box::new(rhs));
    // let mut a = split("asdasd asdasd").unwrap().into_iter();
    Compiler::new().compile(Shlex::new("ls || ls -al || balls").peekable()).unwrap().eval();
    println!("Hello, world!");
}
