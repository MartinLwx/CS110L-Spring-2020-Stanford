use nix::sys::ptrace;
use nix::sys::signal;
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use std::collections::HashMap;
use std::process::Child;
use std::process::Command;
use std::os::unix::process::CommandExt;
use crate::dwarf_data::DwarfData;
use std::mem::size_of;
use crate::debugger::Breakpoint;

pub enum Status {
    /// Indicates inferior stopped. Contains the signal that stopped the process, as well as the
    /// current instruction pointer that it is stopped at.
    Stopped(signal::Signal, usize),

    /// Indicates inferior exited normally. Contains the exit status code.
    Exited(i32),

    /// Indicates the inferior exited due to a signal. Contains the signal that killed the
    /// process.
    Signaled(signal::Signal),
}

/// This function calls ptrace with PTRACE_TRACEME to enable debugging on a process. You should use
/// pre_exec with Command to call this in the child process.
fn child_traceme() -> Result<(), std::io::Error> {
    ptrace::traceme().or(Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "ptrace TRACEME failed",
    )))
}

fn align_addr_to_word(addr: usize) -> usize {
    addr & (-(size_of::<usize>() as isize) as usize)
}


pub struct Inferior {
    child: Child,
}

impl Inferior {
    /// Attempts to start a new inferior process. Returns Some(Inferior) if successful, or None if
    /// an error is encountered.
    pub fn new(target: &str, args: &Vec<String>, breakpoints: &mut HashMap<usize, Breakpoint>) -> Option<Inferior> {
        // spawn a child process running our target program
        let mut cmd = Command::new(target);
        cmd.args(args);

        unsafe {
            cmd.pre_exec(child_traceme);
        }
        
        // Milestone 1: Run the inferior
        let child = cmd.spawn().ok()?;
        let mut inferior = Inferior { child };
        match inferior.wait(None) {
            Ok(_) => {
                // after you wait for SIGTRAP (indicating that the inferior has fully loaded) but before returning
                // , you should install these breakpoints in the child process.
                for bp in breakpoints {
                    let orig_byte = inferior.write_byte(bp.0.clone(), 0xcc).expect("write_byte failed");
                    bp.1.orig_byte = orig_byte; // also, remember the orig_byte
                }

                Some(inferior)
            }
            _ => None,
        }
    }

    /// Returns the pid of this inferior.
    pub fn pid(&self) -> Pid {
        nix::unistd::Pid::from_raw(self.child.id() as i32)
    }

    /// Calls waitpid on this inferior and returns a Status to indicate the state of the process
    /// after the waitpid call.
    pub fn wait(&self, options: Option<WaitPidFlag>) -> Result<Status, nix::Error> {
        Ok(match waitpid(self.pid(), options)? {
            WaitStatus::Exited(_pid, exit_code) => Status::Exited(exit_code),
            WaitStatus::Signaled(_pid, signal, _core_dumped) => Status::Signaled(signal),
            WaitStatus::Stopped(_pid, signal) => {
                let regs = ptrace::getregs(self.pid())?;
                Status::Stopped(signal, regs.rip as usize)
            }
            other => panic!("waitpid returned unexpected status: {:?}", other),
        })
    }

    // Milestone 1: Run the inferior
    /// Wakes up the inferior and waits until it stops or terminates
    pub fn run(&mut self, breakpoints: &mut HashMap<usize, Breakpoint>)-> Result<Status, nix::Error> {
        // Note: we should be able to use `break *addr` even after the inferio has started running
        let clone_bps = breakpoints.clone();
        for bp in clone_bps{
            self.write_byte(bp.0.clone(), 0xcc).expect("write_byte failed");
            // we only reinstall the breakpoints, so we don't need to remember the orig_byte again
        }

        let mut reg_vals = ptrace::getregs(self.pid())?;
        // if inferior stopped at a breakpoint (i.e. (%rip - 1) matches a breakpoint address):
        // use .get() to check if (%rip - 1) matches a breakpoint address
        if let Some(bp) = breakpoints.get(&((reg_vals.rip - 1) as usize)) {
            // restore the first byte of the instruction we replaced
            self.write_byte(bp.addr, bp.orig_byte).unwrap();
            // set %rip = %rip - 1 to rewind the instruction pointer
            reg_vals.rip -= 1;
            ptrace::setregs(self.pid(), reg_vals).unwrap();

            // Note: the breakpoint is no longer in the code, so we need to reset the breakpoint
            // ptrace::step to go to next instruction
            ptrace::step(self.pid(), None).unwrap();
            // wait for inferior to stop due to SIGTRAP, return if the inferior terminates here
            match self.wait(None) {
                Ok(Status::Exited(exit_code)) => return Ok(Status::Exited(exit_code)),
                Ok(Status::Signaled(signal)) => return Ok(Status::Signaled(signal)),
                Ok(Status::Stopped(_, _)) => {
                    // restore the breakpoint
                    // self.write_byte(bp.addr, 0xcc).unwrap();
                }
                Err(e) => return Err(e),
            }
        }

        ptrace::cont(self.pid(), None)?;
        self.wait(None)
    }

    // Milestone 2. Stopping, resuming, and restarting the inferior
    /// Kill the inferior && reap the killed process
    pub fn kill(&mut self) {
       self.child.kill().expect("Child is not running");  // kill existing inferior
       self.wait(None).unwrap(); // reap the killed process
       println!("Killing running inferior (pid {})", self.pid());
    }

    // Milestone 3: Printing a backtrace
    pub fn print_backtrace(&self, debug_data: &DwarfData) -> Result<(), nix::Error> {
        let reg_vals = ptrace::getregs(self.pid())?;
        let (mut instruction_ptr, mut base_ptr) = (reg_vals.rip as usize, reg_vals.rbp as usize);
        loop {
            let lineno = DwarfData::get_line_from_addr(debug_data, instruction_ptr).unwrap();
            let func_name = DwarfData::get_function_from_addr(debug_data, instruction_ptr).unwrap();
            println!("{} ({})", func_name, lineno);
            if func_name == "main" {
                break;
            }
            instruction_ptr = ptrace::read(self.pid(), (base_ptr + 8) as ptrace::AddressType)? as usize;
            base_ptr = ptrace::read(self.pid(), base_ptr as ptrace::AddressType)? as usize;
        }
        Ok(())
    }

    fn write_byte(&mut self, addr: usize, val: u8) -> Result<u8, nix::Error> {
        let aligned_addr = align_addr_to_word(addr);
        let byte_offset = addr - aligned_addr;
        let word = ptrace::read(self.pid(), aligned_addr as ptrace::AddressType)? as u64;
        let orig_byte = (word >> 8 * byte_offset) & 0xff;
        let masked_word = word & !(0xff << 8 * byte_offset);
        let updated_word = masked_word | ((val as u64) << 8 * byte_offset);
        ptrace::write(
            self.pid(),
            aligned_addr as ptrace::AddressType,
            updated_word as *mut std::ffi::c_void,
        )?;
        Ok(orig_byte as u8)
    }
}
