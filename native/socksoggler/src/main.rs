extern crate serde_json;
extern crate shell_words;

use serde_json::Value;
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
        let command_string =
            String::from_utf8(stdin_buf.clone()).expect("stdin was not valid UTF-8 encoded text");
        stdin_buf.clear();

        let command: Value = serde_json::from_str(&command_string)
            .expect("Couldn't deserialize command; is it valid JSON?");

        let to_send = match command["action"].as_str().expect("action is not a string") {
            "off" => ProcessCommand::Off,
            "on" => {
                let args_strs =
                    shell_words::split(command["cmd"].as_str().expect("cmd is not a string"))
                        .expect("Couldn't split command into separate arguments");
                if args_strs.len() < 1 {
                    eprintln!("cmd must be nonempty");
                    continue;
                }

                let mut cmd = Command::new(&args_strs[0]);
                cmd.args(&args_strs[1..]);

                ProcessCommand::On(cmd)
            }
            _ => {
                eprintln!("on and off are the only valid actions");
                continue;
            }
        };
        tx.send(to_send)
            .expect("process manager channel should never be closed");
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
