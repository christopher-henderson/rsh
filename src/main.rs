use bumpalo::Bump;
use shlex::Shlex;
use std::iter::Peekable;
use std::process::Stdio;

mod errors;
mod compiler;
mod process;
mod physical;
use errors::*;
use physical::*;

#[allow(unused)]
pub struct Compiler<'a> {
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

    pub fn compile(&'a mut self, mut lexer: Lexer) -> InterpreterResult<Option<Program<'a>>> {
        Ok(self.compile_statement_list(&mut lexer)?)
    }

    pub fn compile_statement_list(&'a self, lexer: &mut Lexer) -> InterpreterResult<Option<StatementList<'a>>> {
        if lexer.peek().is_none() {
            return Ok(None);
        }
        Ok(Some(StatementList {
            stmt: self.compile_statement(lexer)?,
            smts: match self.compile_statement_list(lexer)? {
                None => None,
                Some(stmts) => Some(Box::new(stmts)),
            }
        }))
    }

    pub fn compile_statement(&'a self, lexer: &mut Lexer) -> InterpreterResult<Box<&'a mut dyn Statement>> {
        let mut tokens = vec![];
        while let Some(token) = lexer.next() {
            match token.as_str() {
                ";" | "\n" => break,
                "&&" => return Ok(And::new(&self.arena, Command::new(&self.arena, tokens)?, self.compile_statement(lexer)?)),
                "||" => return Ok(Or::new(&self.arena, Command::new(&self.arena, tokens)?, self.compile_statement(lexer)?)),
                "|" => return Ok(Pipe::new(&self.arena, Command::new(&self.arena, tokens)?, self.compile_statement(lexer)?)),
                _ => tokens.push(token.clone()),
            }
        }
        return Command::new(&self.arena, tokens);
    }
}
pub type Program<'a> = StatementList<'a>;

pub struct StatementList<'a> {
    stmt: Box<&'a mut dyn Statement>,
    smts: Option<Box<StatementList<'a>>>,
}

impl Statement for StatementList<'_> {
    fn eval(&mut self) -> InterpreterResult<Box<dyn Process>> {
        let mut p = self.stmt.eval()?;
        p.wait();
        if let Some(list) = self.smts.as_mut() {
            list.eval()
        } else {
            Ok(p)
        }
    }

    fn set_stdin(&mut self, stdin: Stdio) {
        self.stmt.set_stdin(stdin);
    }

    fn set_stdout(&mut self, stdout: fn() -> Stdio) {
        self.stmt.set_stdout(stdout)
    }
}

struct Command {
    inner: std::process::Command
}

impl Command {
    fn new(arena: & Bump, mut tokens: Vec<String>) -> InterpreterResult<Box<& mut dyn Statement>> {
        match tokens.iter().map(|t| t.as_str()).collect::<Vec<&str>>().as_slice() {
            [] => Err(InterpreterError{message: "unexpected EOF".to_string()}),
            ["cd"] =>  Ok(CD::new(arena, None)),
            ["cd", path] => Ok(CD::new(arena, Some(path.to_string()))),
            [..] => {
                let mut inner = std::process::Command::new(tokens.remove(0));
                inner.args(tokens);
                Ok(Box::new(arena.alloc(Command{inner})))
            }
        }
    }
}

impl Statement for Command {
    fn eval(&mut self) -> InterpreterResult<Box<dyn Process>> {
        Ok(Box::new(CommandProcess{ child: self.inner.spawn()?, result: None }))
    }
    fn set_stdin(&mut self, stdin: Stdio) {
        self.inner.stdin(stdin);
    }

    fn set_stdout(&mut self, stdout: fn() -> Stdio) {
        self.inner.stdout(stdout());
    }
}

struct And<'a> {
    lhs: Box<&'a mut dyn Statement>,
    rhs: Box<&'a mut dyn Statement>,
}

impl <'a> And<'a> {
    fn new(arena: &'a Bump, lhs: Box<&'a  mut dyn Statement>, rhs: Box<&'a mut dyn Statement>) -> Box<&'a mut dyn Statement> {
        return Box::new(arena.alloc(And{lhs, rhs}))
    }
}

impl Statement for And<'_> {
    fn eval(&mut self) -> InterpreterResult<Box<dyn Process>> {
        if !self.lhs.eval()?.wait() {
            return Err(InterpreterError{message:"".to_string()});
        }
        self.rhs.eval()
    }
    fn set_stdin(&mut self, stdin: Stdio) {
        self.lhs.set_stdin(stdin);
    }
    fn set_stdout(&mut self, stdout: fn() -> Stdio) {
        self.rhs.set_stdout(stdout);
    }
}

struct Or<'a> {
    lhs: Box<&'a mut dyn Statement>,
    rhs: Box<&'a mut dyn Statement>,
}

impl <'a> Or<'a> {
    fn new(arena: &'a Bump, lhs: Box<&'a mut dyn Statement>, rhs: Box<&'a mut dyn Statement>) -> Box<&'a mut dyn Statement> {
        return Box::new(arena.alloc(Or{lhs, rhs}))
    }
}

impl Statement for Or<'_> {
    fn eval(&mut self) -> InterpreterResult<Box<dyn Process>> {
        match self.lhs.eval() {
            Ok(mut p) => { if p.wait() { return Ok(p) } }
            Err(err) => eprintln!("{}", err)
        }
        self.rhs.eval()
    }
    fn set_stdin(&mut self, stdin: Stdio) {
        self.lhs.set_stdin(stdin);
    }
    fn set_stdout(&mut self, stdout: fn() -> Stdio) {
        self.lhs.set_stdout(stdout);
        self.rhs.set_stdout(stdout);
    }
}

#[allow(unused)]
struct Pipe<'a> {
    lhs: Box<&'a mut dyn Statement>,
    rhs: Box<&'a mut dyn Statement>,
    stdout: Stdio
}

impl <'a> Pipe<'a> {
    fn new(arena: &'a Bump, lhs: Box<&'a mut dyn Statement>, rhs: Box<&'a mut dyn Statement>) -> Box<&'a mut dyn Statement> {
        let stdout = Stdio::inherit();
        return Box::new(arena.alloc(Pipe{lhs, rhs, stdout}))
    }
}


impl Statement for Pipe<'_> {
    fn eval(&mut self) -> InterpreterResult<Box<dyn Process>> {
        self.lhs.set_stdout(Stdio::piped);
        self.rhs.set_stdin(self.lhs.eval()?.get_stdout());
        self.rhs.eval()
    }

    fn set_stdin(&mut self, stdin: Stdio) {
        self.lhs.set_stdin(stdin);
    }

    fn set_stdout(&mut self, stdout: fn() -> Stdio) {
        self.rhs.set_stdout(stdout)
    }
}

struct CD {
    target: Option<String>,
}

impl <'a> CD {
    fn new(arena: &'a Bump, target: Option<String>) -> Box<&'a mut dyn Statement> {
        return Box::new(arena.alloc(CD{target}))
    }
}

impl Statement for CD {
    fn eval(&mut self) -> InterpreterResult<Box<dyn Process>> {
       let mut p = CDProcess{ target: self.target.clone(), result: None };
        if !p.wait() {
            return Err(InterpreterError{message: "".to_string()})
        }
        Ok(Box::new(p))
    }

    fn set_stdin(&mut self, _: Stdio) {}

    fn set_stdout(&mut self, _: fn() -> Stdio) {}
}

type Lexer<'a> = Peekable<Shlex<'a>>;

pub trait Statement {
    fn eval(&mut self) -> InterpreterResult<Box<dyn Process>>;
    fn set_stdin(&mut self, stdin: Stdio);
    fn set_stdout(&mut self, stdout: fn() -> Stdio);
    // fn set_stdin(&mut self) -> ()
}


fn main() {
    match Compiler::new().compile(Shlex::new("cd && cd .. || ls").peekable()).unwrap().unwrap().eval() {
        Ok(mut p) => {p.wait();},
        Err(err) => {eprintln!("{}", err);}
    };
}
