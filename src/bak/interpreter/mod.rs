use crate::errors::EvalResult;
use std::process::Stdio;

trait Process {
    fn wait(self) -> EvalResult<bool>;
    fn get_stdout(&self) -> EvalResult<Stdio>;
}

trait Evaluator {
    fn set_stdin(&mut self, stdin: Stdio);
    fn eval(self) -> EvalResult<Box<dyn Process>>;
}

