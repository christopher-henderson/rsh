// use std::process::{Stdio, ChildStdout};
// use std::ffi::OsStr;
//
// pub struct Process {
//     stdin: Stdio,
//     stdout: Stdio,
//     stderr: Stdio,
//     cmd: std::process::Command,
// }
//
// impl Process {
//     pub fn new<T: AsRef<OsStr>>(cmd: T, args: &[T]) -> Process {
//         let mut cmd = std::process::Command::new(cmd);
//         cmd.args(args);
//         Process{
//             stdin: Stdio::inherit(),
//             stdout: Stdio::inherit(),
//             stderr: Stdio::inherit(),
//             cmd: cmd
//         }
//     }
//
//     pub fn spawn(mut self) -> Child {
//         Child::Command(self.cmd.stdin(self.stdin).stdout(self.stdout).stderr(self.stderr).spawn().unwrap())
//     }
//
//     pub fn pipe(&mut self) {
//         self.stdout = Stdio::piped();
//     }
//
//     pub fn set_stdin<T: Into<Stdio>>(&mut self, stdin: Option<T>) {
//         match stdin {
//             None => self.stdin = Stdio::null(),
//             Some(stdin) => self.stdin = stdin.into()
//         };
//     }
// }
//
// pub enum Child {
//     Command(std::process::Child),
//     CD(bool),
// }
//
// impl Child {
//     pub fn get_stdout(self) -> Option<ChildStdout> {
//         match self {
//             Child::Command(child) => child.stdout,
//             Child::CD(_) => None
//         }
//     }
//
//     pub fn wait(self) -> bool {
//         match self {
//             Child::Command(mut child) => match child.wait() {
//                 Err(_) => false,
//                 Ok(status) => status.success()
//             }
//             Child::CD(succeeded) => succeeded
//         }
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn pipe() {
//         let mut ls = Process::new("ls", &vec![]);
//         let mut grep = Process::new("grep", &vec!["LICENSE"]);
//         ls.pipe();
//         grep.set_stdin(ls.spawn().get_stdout());
//         grep.spawn().wait();
//     }
//
//     #[test]
//     fn and() {
//         let mut ls = Process::new("ls", &vec![]);
//         let mut ls_al = Process::new("ls", &vec!["-al"]);
//         if ls.spawn().wait() {
//             ls_al.spawn().wait();
//         }
//     }
//
//     #[test]
//     fn testwait() {
//         // let mut ls = std::process::Command::new("ls");
//         // ls.stdout(Stdio::piped());
//         // let mut child = ls.spawn().unwrap();
//         // child.wait().unwrap();
//         // let mut grep = std::process::Command::new("grep");
//         // grep.arg("LICENSE");
//         // grep.stdin(child.stdout.as_ref().unwrap());
//         // grep.spawn().unwrap().wait();
//     }
// }