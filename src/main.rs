use bumpalo::Bump;
use shlex::Shlex;
use std::iter::Peekable;
use std::process::Stdio;

mod errors;
mod compiler;
mod physical;
use errors::*;
use physical::*;
use std::fs::OpenOptions;
use std::collections::HashMap;
use std::env::Vars;

pub struct Interpreter {
    arena: bumpalo::Bump,
    environment: Environment
}

type Lexer<'a> = Peekable<Shlex<'a>>;

impl <'a> Interpreter {

    pub fn new() -> Self {
        Interpreter { arena: Bump::new(), environment: std::env::vars().collect() }
    }

    pub fn interpret<I: AsRef<str>>(&'a mut self, input: I) -> InterpreterResult<bool> {
        self.arena.reset();
        let mut lexer = Shlex::new(input.as_ref()).peekable();
        let mut ast = self.compile(lexer)?;
        Ok(ast.eval(&mut self.environment)?.wait())
        // Ok(self.compile_statement(Shlex::new(input.as_ref()).peekable())?.eval(&mut self.environment)?.wait())

    }

    fn compile(&'a self, mut lexer: Lexer) -> InterpreterResult<ArenaStatement<'a>> {
        let mut tokens = vec![];
        while let Some(token) = lexer.next() {
            match token.as_str() {
                "&&" => return Ok(self.alloc(And::new(self.compile_expression(tokens)?, self.compile(lexer)?))),
                "||" => return Ok(self.alloc(Or::new(self.compile_expression(tokens)?, self.compile(lexer)?))),
                "|" => return Ok(self.alloc(Pipe::new(self.compile_expression(tokens)?, self.compile(lexer)?))),
                _ => tokens.push(token.clone()),
            }
        }
        return self.compile_expression(tokens);
    }

    fn compile_expression(&'a self, tokens: Vec<String>) -> InterpreterResult<ArenaStatement<'a>> {
        match tokens.iter().map(|t| t.as_str()).collect::<Vec<&str>>().as_slice() {
            [] => {
                Err(InterpreterError{message: "unexpected EOF".to_string()})
            },
            [_, ">"] => {
                Err(InterpreterError{message: "unexpected EOF".to_string()})
            },
            [_, ">>"] => {
                Err(InterpreterError{message: "unexpected EOF".to_string()})
            },
            [">"] => {
                Err(InterpreterError{message: "unexpected EOF".to_string()})
            },
            [">>"] => {
                Err(InterpreterError{message: "unexpected EOF".to_string()})
            },
            ["cd"] =>  {
                Ok(self.alloc(CD::new(None)))
            },
            ["cd", path] => {
                Ok(self.alloc(CD::new(Some(path.to_string()))))
            },
            ["export", kvs@..] => Ok(self.alloc(NOOP{})),
            [head@.., ">", target] => {
                Ok(self.alloc(Redirect::new(Command::new(head), RedirectType::Truncate, target.to_string())))
            },
            [head@.., ">>", target] => {
                Ok(self.alloc(Redirect::new(Command::new(head), RedirectType::Append, target.to_string())))
            },
            [command@..] => {
                Ok(self.alloc(Command::new(command)))
            }
        }
    }

    fn alloc<T: Statement + 'a>(&'a self, val: T) -> ArenaStatement<'a> {
        Box::new(self.arena.alloc(val))
    }

    fn update_environment() {

    }
}

struct NOOP {}

impl Statement for NOOP {
    fn eval(&mut self, _: &mut Environment) -> InterpreterResult<Box<dyn Process>> { Ok(Box::new(NOOP{})) }
    fn set_stdin(&mut self, _: Stdio) {}
    fn set_stdout(&mut self, _: fn() -> Stdio) {}
}

impl Process for NOOP {
    fn get_stdout(self: Box<Self>) -> Stdio { Stdio::null() }
    fn wait(&mut self) -> bool { true }
}

type Environment = HashMap<String, String>;

struct Export {
    environment: Environment
}

impl Export {
    pub fn new(pairs: &[&str]) -> InterpreterResult<Export> {
        let mut export = Export{ environment: Environment::with_capacity(pairs.len()) };
        for pair in pairs {
            if pair.starts_with('=') {
                return Err(InterpreterError{message:format!("export: '{}`: not a valid identifier", pair)})
            }
            let mut kv: Vec<String> = pair.split('=').map(str::to_string).collect();
            export.environment.insert(kv.remove(0), kv.pop().unwrap_or("".to_string()));
        }
        Ok(export)
    }
}

impl Statement for Export {
    fn eval(&mut self, env: &mut Environment) -> InterpreterResult<Box<dyn Process>> {
        Ok(Box::new(NOOP{}))
    }
    fn set_stdin(&mut self, _: Stdio) {}
    fn set_stdout(&mut self, _: fn() -> Stdio) {}
}

pub type Program<'a> = ArenaStatement<'a>;

struct Command { inner: std::process::Command }

impl Command {
    fn new<S: ToString, T: IntoIterator<Item=S>>(tokens: T) -> Command {
        let mut tokens: Vec<String> = tokens.into_iter().map(|i|i.to_string()).collect();
        let mut inner = std::process::Command::new(tokens.remove(0));
        inner.args(tokens);
        Command{inner}
    }
}

impl Statement for Command {
    fn eval(&mut self, env: &mut Environment) -> InterpreterResult<Box<dyn Process>> {
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
    lhs: ArenaStatement<'a>,
    rhs: ArenaStatement<'a>,
}

impl <'a> And<'a> {
    fn new(lhs: ArenaStatement<'a>, rhs: ArenaStatement<'a>) -> And<'a> {
        return And{lhs, rhs}
    }
}

impl Statement for And<'_> {
    fn eval(&mut self, env: &mut Environment) -> InterpreterResult<Box<dyn Process>> {
        if !self.lhs.eval(env)?.wait() {
            return Err(InterpreterError{message:"".to_string()});
        }
        self.rhs.eval(env)
    }
    fn set_stdin(&mut self, stdin: Stdio) {
        self.lhs.set_stdin(stdin);
    }
    fn set_stdout(&mut self, stdout: fn() -> Stdio) {
        self.rhs.set_stdout(stdout);
    }
}

struct Or<'a> {
    lhs: ArenaStatement<'a>,
    rhs: ArenaStatement<'a>,
}

impl <'a> Or<'a> {
    fn new(lhs: ArenaStatement<'a>, rhs: ArenaStatement<'a>) -> Or<'a> {
        return Or{lhs, rhs}
    }
}

impl Statement for Or<'_> {
    fn eval(&mut self, env: &mut Environment) -> InterpreterResult<Box<dyn Process>> {
        match self.lhs.eval(env) {
            Ok(mut p) => { if p.wait() { return Ok(p) } }
            Err(err) => eprintln!("{}", err)
        }
        self.rhs.eval(env)
    }
    fn set_stdin(&mut self, stdin: Stdio) {
        self.lhs.set_stdin(stdin);
    }
    fn set_stdout(&mut self, stdout: fn() -> Stdio) {
        self.lhs.set_stdout(stdout);
        self.rhs.set_stdout(stdout);
    }
}

struct Pipe<'a> {
    lhs: ArenaStatement<'a>,
    rhs: ArenaStatement<'a>,
}

impl <'a> Pipe<'a> {
    fn new(lhs: ArenaStatement<'a>, rhs: ArenaStatement<'a>) -> Pipe<'a> {
        return Pipe{lhs, rhs}
    }
}

impl Statement for Pipe<'_> {
    fn eval(&mut self, env: &mut Environment) -> InterpreterResult<Box<dyn Process>> {
        self.lhs.set_stdout(Stdio::piped);
        self.rhs.set_stdin(self.lhs.eval(env)?.get_stdout());
        self.rhs.eval(env)
    }
    fn set_stdin(&mut self, stdin: Stdio) {
        self.lhs.set_stdin(stdin);
    }
    fn set_stdout(&mut self, stdout: fn() -> Stdio) {
        self.rhs.set_stdout(stdout)
    }
}

struct Redirect {
    cmd: Command,
    target: String,
    _type: RedirectType
}

#[derive(Clone, Copy)]
enum RedirectType {
    Append,
    Truncate
}

impl Into<bool> for RedirectType {
    fn into(self) -> bool {
        match self {
            RedirectType::Append => false,
            RedirectType::Truncate => true
        }
    }
}

type ArenaStatement<'a> = Box<&'a mut dyn Statement>;

impl Redirect {
    fn new(cmd: Command, _type: RedirectType, target: String) -> Redirect {
        return Redirect{cmd, _type, target}
    }
}

impl Statement for Redirect {
    fn eval(&mut self, env: &mut Environment) -> InterpreterResult<Box<dyn Process>> {
        let target = physical::expand(&self.target);
        let mut opts = OpenOptions::new();
        opts.write(true).create(true);
        match self._type {
            RedirectType::Append => opts.append(true),
            RedirectType::Truncate => opts.truncate(true)
        };
        let file = opts.open(target)?;
        self.cmd.inner.stdout(file);
        self.cmd.eval(env)
    }
    fn set_stdin(&mut self, stdin: Stdio) {
        self.cmd.inner.stdin(stdin);
    }
    fn set_stdout(&mut self, _: fn() -> Stdio) {}
}

struct CD {
    target: Option<String>,
}

impl CD {
    fn new(target: Option<String>) -> CD {
        return CD{target}
    }
}

impl Statement for CD {
    fn eval(&mut self, env: &mut Environment) -> InterpreterResult<Box<dyn Process>> {
       let mut p = CDProcess{ target: self.target.clone(), result: None };
        if !p.wait() {
            return Err(InterpreterError{message: "".to_string()})
        }
        Ok(Box::new(p))
    }

    fn set_stdin(&mut self, _: Stdio) {}

    fn set_stdout(&mut self, _: fn() -> Stdio) {}
}

pub trait Statement {
    fn eval(&mut self, env: &mut Environment) -> InterpreterResult<Box<dyn Process>>;
    fn set_stdin(&mut self, stdin: Stdio);
    fn set_stdout(&mut self, stdout: fn() -> Stdio);
}


fn main() {
    // match Compiler::new().compile(Shlex::new("cd && cd .. && ls | rg chris").peekable()).unwrap().unwrap().eval() {
    //     Ok(mut p) => {p.wait();},
    //     Err(err) => {eprintln!("{}", err);}
    // };
    let mut i = Interpreter::new();
    i.interpret("ls | rg src > ls && whoami && pwds || ls").unwrap();
    i.interpret("ls -l && ls -z || ls -a").unwrap();
    // println!("{:?}", shlex::split("ls ; ls"));
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn aasdasd() {
        println!("{:?}", "a=b".split('=').map(str::to_string).collect::<Vec<String>>());
        println!("{:?}", "a".split('=').map(str::to_string).collect::<Vec<String>>());
        println!("{:?}", "a=".split('=').map(str::to_string).collect::<Vec<String>>());
        println!("{:?}", "=a".split('=').map(str::to_string).collect::<Vec<String>>());
        println!("{:?}", "=".split('=').map(str::to_string).collect::<Vec<String>>());
        println!("{:?}", "".split('=').map(str::to_string).collect::<Vec<String>>());
    }
}