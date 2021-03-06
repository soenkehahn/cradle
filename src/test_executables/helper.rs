use std::{
    io::{self, Read, Write},
    path::PathBuf,
    thread::sleep,
    time::Duration,
};

fn main() {
    let mut args = std::env::args();
    args.next().unwrap();
    match args.next().unwrap().as_str() {
        "invalid utf-8 stdout" => io::stdout().write_all(&[0x80]).unwrap(),
        "invalid utf-8 stderr" => io::stderr().write_all(&[0x80]).unwrap(),
        "exit code 42" => std::process::exit(42),
        "stream chunk then wait for file" => {
            println!("foo");
            io::stdout().flush().unwrap();
            let file = PathBuf::from("./file");
            while !file.exists() {
                sleep(Duration::from_secs_f32(0.1));
            }
        }
        "output foo and exit with 42" => {
            println!("foo");
            std::process::exit(42)
        }
        "write to stderr" => {
            eprintln!("foo");
        }
        "write to stderr and exit with 42" => {
            eprintln!("foo");
            std::process::exit(42)
        }
        "stream chunk to stderr then wait for file" => {
            eprintln!("foo");
            let file = PathBuf::from("./file");
            while !file.exists() {
                sleep(Duration::from_secs_f32(0.1));
            }
        }
        "reverse" => {
            let mut input = Vec::new();
            io::stdin().read_to_end(&mut input).unwrap();
            input.reverse();
            io::stdout().write_all(&input).unwrap();
            io::stdout().flush().unwrap();
        }
        "wait until stdin is closed" => {
            while !stdin_is_closed() {}
            println!("stdin is closed");
        }
        "echo" => {
            for variable in args {
                match std::env::var(&variable).unwrap().as_str() {
                    "" => println!("empty variable: {}", variable),
                    value => println!("{}", value),
                }
            }
        }
        arg => panic!("cradle_test_helper: invalid arg: {}", arg),
    }
}

fn stdin_is_closed() -> bool {
    #[cfg(unix)]
    {
        use nix::poll::{poll, PollFd, PollFlags};
        let mut poll_fds = [PollFd::new(0, PollFlags::all())];
        poll(&mut poll_fds, 0).unwrap();
        if let Some(events) = poll_fds[0].revents() {
            events.contains(PollFlags::POLLHUP)
        } else {
            false
        }
    }
    #[cfg(windows)]
    panic!("stdin_is_closed is not supported on windows")
}
