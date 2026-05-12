use anteater_core::error::{Error, Result};
use anteater_core::process::{Process, ProcessState, StopReason};
use nix::unistd::Pid;
use rustyline::DefaultEditor;
use std::path::Path;

fn attach() -> Result<Process> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 3 && args[1] == "-p" {
        let raw: i32 = args[2].parse().unwrap_or(-1);
        let pid = Pid::from_raw(raw);
        Process::attach(pid)
    } else {
        Process::launch(Path::new(&args[1]), true)
    }
}

fn handle_command(process: &mut Process, line: &str) -> Result<()> {
    let args: Vec<&str> = line.split_whitespace().collect();
    let command = args[0];
    if "continue".starts_with(command) {
        process.resume()?;
        let reason = process.wait_on_signal()?;
        print_stop_reason(process.pid, &reason);
        Ok(())
    } else {
        Err(Error::InvalidArg("Unknown command".into()))
    }
}

fn print_stop_reason(pid: Pid, reason: &StopReason) {
    print!("Process {} ", pid);
    match reason.state {
        ProcessState::Exited => println!("exited with status {}", reason.info),
        ProcessState::Terminated => println!("terminated with signal {}", reason.info),
        ProcessState::Stopped => println!("stopped with signal {}", reason.info),
        _ => println!("Unknown stop reason"),
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        return Err(Error::InvalidArg("No arguments given".into()));
    }
    let mut process = attach()?;
    let mut rl = DefaultEditor::new()?;
    let mut last_cmd = String::new();

    while let Ok(line) = rl.readline("sdb> ") {
        let cmd = if line.is_empty() {
            last_cmd.clone()
        } else {
            rl.add_history_entry(&line)?;
            last_cmd = line.clone();
            line
        };

        if !cmd.is_empty() {
            handle_command(&mut process, &cmd)?;
        }
    }
    Ok(())
}
