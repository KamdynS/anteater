use crate::error::{Error, Result};
use crate::register_info::{RegisterFormat, RegisterInfo, RegisterType, DR_IDS};
use crate::registers::Value;

use nix::fcntl::OFlag;
use nix::libc;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::sys::{
    ptrace,
    ptrace::regset,
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
    pub data: libc::user,
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
                    data: unsafe { std::mem::zeroed() },
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
            data: unsafe { std::mem::zeroed() },
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
                self.read_all_registers()?;
                StopReason {
                    state: ProcessState::Stopped,
                    info: sig as i32 as u8,
                }
            }
            _ => todo!("handle other variants if needed"),
        };

        Ok(reason)
    }

    pub fn read_all_registers(&mut self) -> Result<()> {
        self.data.regs = ptrace::getregs(self.pid)?;
        self.data.i387 = ptrace::getregset::<regset::NT_PRFPREG>(self.pid)?;

        self.data
            .u_debugreg
            .iter_mut()
            .zip(DR_IDS.iter())
            .try_for_each(|(slot, &id)| {
                let offset = RegisterInfo::by_id(id)?.offset;
                let data = ptrace::read_user(self.pid, offset as ptrace::AddressType)?;
                *slot = data as u64;
                Ok::<(), Error>(())
            })?;
        Ok(())
    }

    pub fn write_user_area(&mut self, offset: usize, data: u64) -> Result<()> {
        ptrace::write_user(self.pid, offset as ptrace::AddressType, data as i64)?;
        Ok(())
    }

    pub fn write_fprs(&mut self, fprs: libc::user_fpregs_struct) -> Result<()> {
        ptrace::setregset::<regset::NT_PRFPREG>(self.pid, fprs)?;
        Ok(())
    }

    pub fn write_gprs(&mut self, gprs: libc::user_regs_struct) -> Result<()> {
        ptrace::setregs(self.pid, gprs)?;
        Ok(())
    }

    // for read_register
    fn data_bytes(&self) -> &[u8] {
        let ptr = &self.data as *const libc::user as *const u8;
        let len = std::mem::size_of::<libc::user>();
        // SAFETY: libc::user is a POD C struct; every byte is initialized
        // after read_all_registers and there's no niche representation
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }

    // for write_register
    fn data_bytes_mut(&mut self) -> &mut [u8] {
        let ptr = &mut self.data as *mut libc::user as *mut u8;
        let len = std::mem::size_of::<libc::user>();
        // SAFETY: same as data_bytes
        unsafe { std::slice::from_raw_parts_mut(ptr, len) }
    }

    pub fn read_register(&self, info: &RegisterInfo) -> Value {
        let bytes = self.data_bytes();
        let reg_bytes = &bytes[info.offset..info.offset + info.size];
        match (&info.format, info.size) {
            (RegisterFormat::Uint, 1) => {
                let arr = reg_bytes.try_into().unwrap();
                Value::U8(u8::from_ne_bytes(arr))
            }
            (RegisterFormat::Uint, 2) => {
                let arr = reg_bytes.try_into().unwrap();
                Value::U16(u16::from_ne_bytes(arr))
            }
            (RegisterFormat::Uint, 4) => {
                let arr = reg_bytes.try_into().unwrap();
                Value::U32(u32::from_ne_bytes(arr))
            }
            (RegisterFormat::Uint, 8) => {
                let arr = reg_bytes.try_into().unwrap();
                Value::U64(u64::from_ne_bytes(arr))
            }
            (RegisterFormat::DoubleFloat, _) => {
                let arr = reg_bytes.try_into().unwrap();
                Value::Double(f64::from_ne_bytes(arr))
            }
            (RegisterFormat::LongDouble, _) => Value::Byte128(reg_bytes.try_into().unwrap()),
            (RegisterFormat::Vector, 8) => Value::Byte64(reg_bytes.try_into().unwrap()),
            (RegisterFormat::Vector, 16) => Value::Byte128(reg_bytes.try_into().unwrap()),
            _ => panic!(
                "unexpected register layout: {:?} size {}",
                info.format, info.size
            ),
        }
    }

    fn widen(info: &RegisterInfo, val: Value) -> [u8; 16] {
        let mut buf = [0u8; 16];
        match val {
            Value::U8(v) => {
                let bytes = v.to_ne_bytes();
                buf[..1].copy_from_slice(&bytes);
            }
            Value::U16(v) => {
                let bytes = v.to_ne_bytes();
                buf[..2].copy_from_slice(&bytes);
            }
            Value::U32(v) => {
                let bytes = v.to_ne_bytes(); // [u8; 4]
                buf[..4].copy_from_slice(&bytes);
            }
            Value::U64(v) => {
                let bytes = v.to_ne_bytes();
                buf[..8].copy_from_slice(&bytes);
            }
            Value::I8(v) => {
                let extended = (v as i64).to_ne_bytes(); // [u8; 8]
                buf[..info.size].copy_from_slice(&extended[..info.size]);
            }
            Value::I16(v) => {
                let extended = (v as i64).to_ne_bytes(); // [u8; 8]
                buf[..info.size].copy_from_slice(&extended[..info.size]);
            }
            Value::I32(v) => {
                let extended = (v as i64).to_ne_bytes(); // [u8; 8]
                buf[..info.size].copy_from_slice(&extended[..info.size]);
            }
            Value::I64(v) => {
                let extended = (v).to_ne_bytes(); // [u8; 8]
                buf[..info.size].copy_from_slice(&extended[..info.size]);
            }
            Value::Float(v) => {
                let promoted = (v as f64).to_ne_bytes(); // [u8; 8]
                buf[..8].copy_from_slice(&promoted);
            }
            Value::Double(v) => {
                let promoted = (v).to_ne_bytes(); // [u8; 8]
                buf[..8].copy_from_slice(&promoted);
            }
            Value::Byte64(v) => buf[..8].copy_from_slice(&v),
            // Blob: identity. Byte128 IS the buffer.
            Value::Byte128(b) => return b,
        }
        buf
    }
    pub fn write_register(&mut self, info: &RegisterInfo, val: Value) -> Result<()> {
        let buf = Self::widen(info, val);
        let bytes = self.data_bytes_mut();
        bytes[info.offset..info.offset + info.size].copy_from_slice(&buf[..info.size]);
        if info.kind == RegisterType::Fpr {
            self.write_fprs(self.data.i387)?;
        } else {
            let aligned = info.offset & !0b111;
            let word: [u8; 8] = self.data_bytes()[aligned..aligned + 8].try_into().unwrap();
            self.write_user_area(aligned, u64::from_ne_bytes(word))?;
        }

        Ok(())
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
