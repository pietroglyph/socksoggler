use std::fs;
use std::io;
use std::io::BufRead;
use std::process::{Child, Command};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

enum ProcessCommand {
    On(Command),
    Off,
}

// We use "¬" as a delimiter because we can't get a newline into the message
const LINE_DELIMITER: u8 = b'\xAC';

fn main() {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stdin_buf: Vec<u8> = vec![];
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || process_manager_thread(rx));

    loop {
        stdin
            .read_until(LINE_DELIMITER, &mut stdin_buf)
            .expect("Couldn't read from stdin");
        let mut command = String::from_utf8(stdin_buf).expect("stdin was not valid utf-8 encoded text");
        stdin_buf = vec![];
        command.pop(); // Don't include the delimiter

        // Native messaging annoyingly only sends strigified JSON
        // Because we add a newline in the extension js messages
        // look like "message\x1E", which means that quotes will
        // always end up in front.
        command = String::from(command.trim());
        command = String::from(command.trim_start_matches('"'));
        let split = command.split_whitespace();

        let mut cmd_to_run: Option<Command> = None;
        for (i, val) in split.enumerate() {
            if i == 0 && val == "off" {
                tx.send(ProcessCommand::Off)
                    .expect("process manager channel should never be closed");
                continue;
            } else if i == 1 {
                cmd_to_run = Some(Command::new(val));
            } else if i > 1 {
                if let Some(ref mut c) = cmd_to_run {
                    (*c).arg(val);
                }
            }
        }

        if let Some(c) = cmd_to_run {
            tx.send(ProcessCommand::On(c))
                .expect("process manager channel should never be closed");
        }

        fs::write("/home/declan/wazzup", command.clone()).expect("boo");
    }
}

fn process_manager_thread(rx: mpsc::Receiver<ProcessCommand>) -> ! {
    let mut current_command: ProcessCommand = ProcessCommand::Off;
    let mut child_process: Option<Child> = None;
    loop {
        thread::sleep(Duration::from_millis(1));

        current_command = match rx.try_recv() {
            Ok(pc) => pc,
            Err(_) => current_command,
        };

        match current_command {
            ProcessCommand::On(ref mut command) => {
                if is_child_alive(&mut child_process) {
                    continue;
                }
                child_process = Some(command.spawn().expect("Couldn't start command"));
            }
            ProcessCommand::Off => {
                if !is_child_alive(&mut child_process) {
                    continue;
                }
                child_process
                    .expect("Child should never be None because of check in is_child_alive")
                    .kill()
                    .expect("Couldln't kill child process");
                child_process = None;
            }
        };
    }
}

fn is_child_alive(child: &mut Option<Child>) -> bool {
    match child {
        Some(proc) => proc.try_wait().unwrap().is_none(),
        None => return false,
    }
}
