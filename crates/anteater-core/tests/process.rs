use anteater_core::process::Process;
use nix::sys::signal::kill;
use nix::unistd::Pid;
use std::path::{Path, PathBuf};

fn which(prog: &str) -> PathBuf {
    let out = std::process::Command::new("which")
        .arg(prog)
        .output()
        .unwrap();
    PathBuf::from(String::from_utf8(out.stdout).unwrap().trim())
}

fn process_status(pid: Pid) -> char {
    let data = std::fs::read_to_string(format!("/proc/{}/stat", pid)).unwrap();
    let last_paren = data.rfind(')').unwrap();
    data.as_bytes()[last_paren + 2] as char
}

#[test]
fn launch_success() {
    let proc = Process::launch(&which("yes"), true).unwrap();
    assert!(kill(proc.pid, None).is_ok());
}

#[test]
fn launch_no_such_program() {
    let result = Process::launch(Path::new("you_do_not_have_to_be_good"), true);
    assert!(result.is_err());
}

#[test]
fn attach_invalid_pid() {
    assert!(Process::attach(Pid::from_raw(0)).is_err());
}

#[test]
fn attach_success() {
    let target = Process::launch(&which("yes"), false).unwrap();
    let _proc = Process::attach(target.pid).unwrap();
    assert_eq!(process_status(target.pid), 't');
}

#[test]
fn resume_launch() {
    let mut proc = Process::launch(&which("yes"), true).unwrap();
    proc.resume().unwrap();
    let s = process_status(proc.pid);
    assert!(s == 'R' || s == 'S');
}

#[test]
fn resume_attach() {
    let target = Process::launch(&which("yes"), false).unwrap();
    let mut proc = Process::attach(target.pid).unwrap();
    proc.resume().unwrap();
    let s = process_status(target.pid);
    assert!(s == 'R' || s == 'S');
}

#[test]
fn resume_already_terminated() {
    let mut proc = Process::launch(&which("true"), true).unwrap();
    proc.resume().unwrap();
    proc.wait_on_signal().unwrap();
    assert!(proc.resume().is_err());
}
