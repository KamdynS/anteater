use crate::error::{Error, Result};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::sys::{
    ptrace,
    signal::{kill, Signal},
};
use nix::unistd::{execvp, fork, ForkResult, Pid};
use std::ffi::CString;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessState {
    Stopped,
    Running,
    Exited,
    Terminated,
}

pub struct StopReason {
    pub state: ProcessState,
    pub info: u8,
}
pub struct Process {
    pub pid: Pid,
    terminate_on_end: bool,
    state: ProcessState,
}

impl Process {
    pub fn launch(path: &Path) -> Result<Self> {
        let raw_file = CString::new(path.to_str().unwrap())?;
        match unsafe { fork() }? {
            ForkResult::Child => {
                ptrace::traceme()?;
                execvp(&raw_file, &[&raw_file])?;
                std::process::exit(1)
            }
            ForkResult::Parent { child } => {
                let mut proc = Process {
                    pid: child,
                    terminate_on_end: true,
                    state: ProcessState::Stopped,
                };
                proc.wait_on_signal()?;
                Ok(proc)
            }
        }
    }

    pub fn attach(pid: Pid) -> Result<Self> {
        if pid == Pid::from_raw(0) {
            return Err(Error::InvalidArg("Invalid PID".into()));
        }
        ptrace::attach(pid)?;
        let mut proc = Process {
            pid: pid,
            terminate_on_end: false,
            state: ProcessState::Stopped,
        };
        proc.wait_on_signal()?;
        Ok(proc)
    }

    pub fn resume(&mut self) -> Result<()> {
        ptrace::cont(self.pid, None)?;
        self.state = ProcessState::Running;
        Ok(())
    }

    pub fn wait_on_signal(&mut self) -> Result<StopReason> {
        let status = waitpid(self.pid, None)?;

        let reason = match status {
            WaitStatus::Exited(_, code) => {
                self.state = ProcessState::Exited;
                StopReason {
                    state: ProcessState::Exited,
                    info: code as u8,
                }
            }
            WaitStatus::Signaled(_, sig, _) => {
                self.state = ProcessState::Terminated;
                StopReason {
                    state: ProcessState::Terminated,
                    info: sig as i32 as u8,
                }
            }
            WaitStatus::Stopped(_, sig) => {
                self.state = ProcessState::Stopped;
                StopReason {
                    state: ProcessState::Stopped,
                    info: sig as i32 as u8,
                }
            }
            _ => todo!("handle other variants if needed"),
        };

        Ok(reason)
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        if self.pid != Pid::from_raw(0) {
            if self.state == ProcessState::Running {
                kill(self.pid, Signal::SIGSTOP).ok();
                waitpid(self.pid, None).ok();
            }
            ptrace::detach(self.pid, Signal::SIGCONT).ok();
            kill(self.pid, Signal::SIGCONT).ok();

            if self.terminate_on_end {
                kill(self.pid, Signal::SIGKILL).ok();
                waitpid(self.pid, None).ok();
            }
        }
    }
}
