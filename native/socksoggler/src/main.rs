use std::io;
use std::process::{Child, Command};
use std::sync::mpsc;
use std::thread;

enum ProcessCommand {
    On(Command),
    Off,
}

fn main() {
    let stdin = io::stdin();
    let mut command = String::new();
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        process_manager_thread(rx);
    });

    loop {
        command.clear();
        match stdin.read_line(&mut command) {
            Ok(_) => {
                let split = command.split_whitespace();

                let mut cmd_to_run: Option<Command> = None;
                for (i, val) in split.enumerate() {
                    if i == 0 && val == "off" {
                        tx.send(ProcessCommand::Off)
                            .expect("process manager mpsc should never be closed");
                        continue;
                    } else if i == 1 {
                        cmd_to_run = Some(Command::new(val));
                    } else if i > 1 {
                        match cmd_to_run {
                            Some(mut c) => {
                                c.arg(val);
                                cmd_to_run = Some(c);
                            }
                            None => (),
                        }
                    }
                }

                cmd_to_run.map(|c| {
                    tx.send(ProcessCommand::On(c))
                        .expect("process manager mpsc should never be closed");
                });
            }
            Err(error) => println!("Unrecognized command: {}", error),
        }
    }
}

fn process_manager_thread(rx: mpsc::Receiver<ProcessCommand>) -> ! {
    let mut child_process: Option<&mut Child> = None;
    let mut must_keep_alive = false;
    loop {
        child_process = match rx.try_recv() {
            Ok(pc) => match pc {
                ProcessCommand::On(mut c) => {
                    must_keep_alive = true;
                    if child_process.is_none() {
                        c.spawn().as_mut().ok()
                    } else {
                        child_process
                    }
                }
                ProcessCommand::Off => {
                    must_keep_alive = false;
                    child_process
                }
            },
            Err(_) => child_process,
        };
        // rx.try_recv().ok().map(|pc| match pc {
        //     ProcessCommand::On(c) => {
        //         must_keep_alive = true;
        //         if child_process.is_none() {
        //             child_process = c.spawn().as_mut().ok();
        //         }
        //     }
        //     ProcessCommand::Off => {
        //         must_keep_alive = false;
        //     }
        // });

        if let Some(c) = child_process {
            if must_keep_alive && c.try_wait().is_err() {
                // Restart the process the next time around
                child_process = None;
                continue;
            } else if !must_keep_alive {
                // We don't care if the child is already dead, so no error handling is needed
                c.kill();
                child_process = None;
                continue;
            }

            child_process = Some(c);
        }

        // let should_start = false;
        // let should_stop = true;
        // rx.try_recv().map(|pc| {
        //     match pc {
        //         On(c) => {
        //             should_start = true;
        //         },
        //         Off => {
        //             should_stop = true;
        //         },
        //     }
        // });
    }
}
