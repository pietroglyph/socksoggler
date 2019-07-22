extern crate shell_words;

use std::convert::TryFrom;
use std::io;
use std::io::Read;
use std::process::{Child, Command};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

enum ProcessCommand {
    On(Command),
    Off,
}

fn main() -> ! {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stdin_buf: Vec<u8> = vec![];
    let mut stdin_len_buf: [u8; 4] = [0; 4];
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || process_manager_thread(rx));

    loop {
        // Every message starts with a 32-bit unsigned integer in native byte order representing message length
        stdin
            .read_exact(&mut stdin_len_buf[..])
            .expect("Couldn't read length field from stdin");
        let len_to_read = usize::try_from(u32::from_ne_bytes(stdin_len_buf))
            .expect("u32 won't fit into a usize!");

        // We must make sure the vec is large enough for the message
        if stdin_buf.len() < len_to_read {
            stdin_buf.resize_with(len_to_read, Default::default);
        }

        stdin
            .read_exact(&mut stdin_buf[..len_to_read])
            .expect("Couldn't read message contents from stdin into buffer");
        let command =
            String::from_utf8(stdin_buf.clone()).expect("stdin was not valid UTF-8 encoded text");
        stdin_buf.clear();

        // We split into shell arguments and remove the surrounding quotation marks
        let split = shell_words::split(&command[1..command.len() - 1])
            .expect("Couldn't split command into separate arguments");

        let mut cmd_to_run: Option<Command> = None;
        for (i, val) in split.iter().enumerate() {
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
                    .expect("Child should never be None because of the check in is_child_alive")
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
