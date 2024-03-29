use crate::debugger_command::DebuggerCommand;
use crate::inferior::{Inferior, Status};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use crate::dwarf_data::{DwarfData, Error as DwarfError};
use std::collections::HashMap;

// Milestone 6: Continuing from breakpoints
#[derive(Clone, Debug)]
pub struct Breakpoint {
    pub addr: usize,
    pub orig_byte: u8,
}

pub struct Debugger {
    target: String,
    history_path: String,
    readline: Editor<()>,
    inferior: Option<Inferior>,
    debug_data: DwarfData,
    breakpoints: HashMap<usize, Breakpoint>,
}

impl Debugger {
    /// Initializes the debugger.
    pub fn new(target: &str) -> Debugger {
        let debug_data = match DwarfData::from_file(target) {
            Ok(val) => val,
            Err(DwarfError::ErrorOpeningFile) => {
                println!("Could not open file {}", target);
                std::process::exit(1);
            }
            Err(DwarfError::DwarfFormatError(err)) => {
                println!("Could not debugging symbols from {}: {:?}", target, err);
                std::process::exit(1);
            }
        };
        debug_data.print();
        // TODO (milestone 3): initialize the DwarfData

        let history_path = format!("{}/.deet_history", std::env::var("HOME").unwrap());
        let mut readline = Editor::<()>::new();
        // Attempt to load history from ~/.deet_history if it exists
        let _ = readline.load_history(&history_path);

        Debugger {
            target: target.to_string(),
            history_path,
            readline,
            inferior: None,
            debug_data,
            breakpoints: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.get_next_command() {
                DebuggerCommand::Run(args) => {
                    // Kill any existing inferiors before starting new ones
                    // , so that there is only one inferior at a time
                    if self.inferior.is_some() {
                        self.inferior.as_mut().unwrap().kill();
                        self.inferior = None;
                    }

                    if let Some(inferior) = Inferior::new(&self.target, &args, &mut self.breakpoints) {
                        // Create the inferior
                        self.inferior = Some(inferior);
                        // TODO (milestone 1): make the inferior run
                        match self.inferior.as_mut().unwrap().run(&mut self.breakpoints).unwrap() {
                            Status::Stopped(signal, instruction_ptr) => {
                                println!("Child stopped (signal {})", signal);
                                if let Some(lineno) = DwarfData::get_line_from_addr(&self.debug_data, instruction_ptr) {
                                    println!("Stopped at {}", lineno);
                                }
                            }
                            Status::Exited(exit_code) => {
                                println!("Child exited (status {})", exit_code);
                                self.inferior = None;
                            }
                            _ => (),
                        }
                    } else {
                        println!("Error starting subprocess");
                    }
                }

                // Milestone 2. Stopping, resuming, and restarting the inferior
                DebuggerCommand::Continue => {
                    // check whether an inferior is running
                    // , and print an error message if there is not one running.
                    if self.inferior.is_none() {
                        println!("No inferior is running");
                        continue
                    }


                    match self.inferior.as_mut().unwrap().run(&mut self.breakpoints).unwrap() {
                        Status::Stopped(signal, instruction_ptr) => {
                            println!("Child stopped (signal {})", signal);
                            if let Some(lineno) = DwarfData::get_line_from_addr(&self.debug_data, instruction_ptr) {
                                println!("Stopped at {}", lineno);
                            }
                        }
                        Status::Exited(exit_code) => {
                            println!("Child exited (status {})", exit_code);
                            self.inferior = None;
                        }
                        _ => (),
                    } 
                }
                
                // Milestone 3: Printing a backtrace
                DebuggerCommand::Backtrace => {
                    self.inferior.as_mut().unwrap().print_backtrace(&self.debug_data).unwrap();
                }

                // Milestone 5: Setting breakpoints
                DebuggerCommand::Breakpoint(bp_addr) => {
                    if bp_addr.starts_with('*') {
                        // Case 1. raw address
                        println!("Set breakpoint {} at {}", self.breakpoints.len(), &bp_addr[1..]);
                        match DebuggerCommand::parse_address(&bp_addr[1..]) {
                            Some(addr) => {
                                self.breakpoints.insert(addr, Breakpoint { addr, orig_byte: 0 });
                                println!("Set breakpoint {} at {:#x}", self.breakpoints.len(), addr);
                            }
                            None => {
                                println!("Please use legal hex number :(");
                                continue;
                            }
                        }
                    // } else if let Some(lineno) = DebuggerCommand::parse_address(&bp_addr)  {
                    } else if let Ok(lineno) = bp_addr.parse() {
                        // Case 2. line number
                        match self.debug_data.get_addr_for_line(None, lineno) {
                            Some(addr) => {
                                self.breakpoints.insert(addr, Breakpoint { addr, orig_byte: 0 });
                                println!("Set breakpoint {} at {:#x}", self.breakpoints.len(), addr);
                            }
                            None => {
                                println!("Please use legal lineno :(");
                                continue;
                            }
                        }
                    } else {
                        // Case 3. function name or none of the cases
                        match self.debug_data.get_addr_for_function(None, &bp_addr) {
                            Some(addr) => {
                                self.breakpoints.insert(addr, Breakpoint { addr, orig_byte: 0 });
                                println!("Set breakpoint {} at {:#x}", self.breakpoints.len(), addr);
                            }
                            None => {
                                println!("Please use legal symbol as the breakpoint :(");
                                continue;
                            }
                        }
                    }
                }

                DebuggerCommand::Quit => {
                    // Kill any existing inferiors before starting new ones
                    // , so that there is only one inferior at a time
                    if self.inferior.is_some() {
                        self.inferior.as_mut().unwrap().kill();
                        self.inferior = None;
                    }
                    return;
                }
            }
        }
    }

    /// This function prompts the user to enter a command, and continues re-prompting until the user
    /// enters a valid command. It uses DebuggerCommand::from_tokens to do the command parsing.
    ///
    /// You don't need to read, understand, or modify this function.
    fn get_next_command(&mut self) -> DebuggerCommand {
        loop {
            // Print prompt and get next line of user input
            match self.readline.readline("(deet) ") {
                Err(ReadlineError::Interrupted) => {
                    // User pressed ctrl+c. We're going to ignore it
                    println!("Type \"quit\" to exit");
                }
                Err(ReadlineError::Eof) => {
                    // User pressed ctrl+d, which is the equivalent of "quit" for our purposes
                    return DebuggerCommand::Quit;
                }
                Err(err) => {
                    panic!("Unexpected I/O error: {:?}", err);
                }
                Ok(line) => {
                    if line.trim().len() == 0 {
                        continue;
                    }
                    self.readline.add_history_entry(line.as_str());
                    if let Err(err) = self.readline.save_history(&self.history_path) {
                        println!(
                            "Warning: failed to save history file at {}: {}",
                            self.history_path, err
                        );
                    }
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    if let Some(cmd) = DebuggerCommand::from_tokens(&tokens) {
                        return cmd;
                    } else {
                        println!("Unrecognized command.");
                    }
                }
            }
        }
    }
}
