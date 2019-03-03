use std::io;

fn main() {
    println!("Starting proxy-toggler command runner native client...");

    let stdin = io::stdin();
    let mut command = String::new();
    loop {
        command.clear();
        match stdin.read_line(&mut command) {
            Ok(n) => {
                println!("Read {} ({} bytes long)", command, n);
            }
            Err(error) => println!("Unrecognized command: {}", error),
        }
    }
}
