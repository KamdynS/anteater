use crate::error::{Error, Result};
use nix::fcntl::OFlag;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::sys::{
    ptrace,
    signal::{kill, Signal},
};
use nix::unistd::{execvp, fork, pipe2, read, write, ForkResult, Pid};
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
    is_attached: bool,
}

impl Process {
    pub fn launch(path: &Path, debug: bool) -> Result<Self> {
        let raw_file = CString::new(path.to_str().unwrap())?;
        let (read_fd, write_fd) = pipe2(OFlag::O_CLOEXEC)?;

        match unsafe { fork() }? {
            ForkResult::Child => {
                drop(read_fd);
                let result = (|| -> Result<()> {
                    if debug {
                        ptrace::traceme()?;
                    }
                    execvp(&raw_file, &[&raw_file])?;
                    Ok(())
                })();
                if let Err(e) = result {
                    let msg = format!("{}", e);
                    let _ = write(&write_fd, msg.as_bytes());
                }
                std::process::exit(1)
            }
            ForkResult::Parent { child } => {
                drop(write_fd);
                let mut buf = [0u8; 1024];
                let n = read(&read_fd, &mut buf)?;
                drop(read_fd);

                if n > 0 {
                    waitpid(child, None).ok();
                    let msg = String::from_utf8_lossy(&buf[..n]).into_owned();
                    return Err(Error::InvalidArg(msg));
                }

                let mut proc = Process {
                    pid: child,
                    terminate_on_end: true,
                    state: ProcessState::Stopped,
                    is_attached: debug,
                };
                if debug {
                    proc.wait_on_signal()?;
                }
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
            pid,
            terminate_on_end: false,
            state: ProcessState::Stopped,
            is_attached: true,
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
            if self.is_attached {
                if self.state == ProcessState::Running {
                    kill(self.pid, Signal::SIGSTOP).ok();
                    waitpid(self.pid, None).ok();
                }
                ptrace::detach(self.pid, Signal::SIGCONT).ok();
                kill(self.pid, Signal::SIGCONT).ok();
            }

            if self.terminate_on_end {
                kill(self.pid, Signal::SIGKILL).ok();
                waitpid(self.pid, None).ok();
            }
        }
    }
}
