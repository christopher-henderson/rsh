use std::process::Stdio;
use std::path::{PathBuf, Component};
use std::ffi::OsStr;

pub trait Process {
    fn get_stdout(self: Box<Self>) -> Stdio;
    fn wait(&mut self) -> bool;
}

pub struct CommandProcess {
    pub(crate) child: std::process::Child,
    pub(crate) result: Option<bool>
}

impl Process for CommandProcess {
    fn get_stdout(self: Box<Self>) -> Stdio {
        self.child.stdout.unwrap().into()
    }

    fn wait(&mut self) -> bool {
        match self.result {
            Some(result) => result,
            None => {
                match self.child.wait() {
                    Ok(result) => {
                        self.result = Some(result.success());
                        self.result.unwrap()
                    }
                    Err(err) => {
                        eprintln!("{}", err);
                        self.result = Some(false);
                        false
                    }
                }
            }
        }
    }
}

pub struct CDProcess {
    pub(crate) target: Option<String>,
    pub(crate) result: Option<bool>
}

impl CDProcess {
    fn expand(&self) -> PathBuf {
        let target = if let Some(target) = &self.target {
            PathBuf::from(target)
        } else {
            return PathBuf::from(dirs::home_dir().unwrap())
        };
        let mut pb = PathBuf::new();
        let home = OsStr::new("~");
        target.components().into_iter().for_each(|component| {
            match component {
                Component::Normal(path) if path == home => pb.push(dirs::home_dir().unwrap()),
                path => pb.push(path)
            }
        });
        pb
    }
}

impl Process for CDProcess {
    fn get_stdout(self: Box<Self>) -> Stdio {
        Stdio::null()
    }

    fn wait(&mut self) -> bool {
        match self.result {
            Some(result) => result,
            None => {
                match std::env::set_current_dir(self.expand()) {
                    Ok(_) => {
                        self.result = Some(true);
                        true
                    }
                    Err(err) => {
                        eprintln!("{}", err);
                        self.result = Some(false);
                        false
                    }
                }
            }
        }
    }
}