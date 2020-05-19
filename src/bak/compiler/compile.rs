use crate::errors::CompilerResult;

pub trait Compile<T> {
    fn compile(source: &mut T) -> CompilerResult<Self> where Self: std::marker::Sized;
}