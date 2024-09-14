use std::{
    io::{BufRead, BufReader, Write},
    os::unix::process::ExitStatusExt,
    process::{Child, Command, Stdio},
    thread::JoinHandle,
};

use clap::Parser;
use crossbeam::channel::bounded;
use nix::libc::SIGTERM;

#[derive(Default, Parser, Debug)]
#[command(version)]
struct Arguments {
    #[arg(default_value_t = false, short, long)]
    kill_others: bool,
    cmd: Vec<String>,
}

fn main() {
    let args = Arguments::parse();
    println!("args: {args:?}");

    spawn_cmds(&args.cmd, args.kill_others);
}

fn spawn_cmds(args: &[String], kill_others: bool) {
    let proceses = args
        .iter()
        .enumerate()
        .map(|(i, arg)| (i, arg.to_string(), spawn_child(i, arg)))
        .collect::<Vec<(usize, String, Child)>>();

    let mut handles: Vec<(u32, JoinHandle<()>)> = vec![];
    let (tx, rx) = bounded(1);

    for (i, cmd, mut process) in proceses {
        let tx = tx.clone();
        handles.push((
            process.id(),
            std::thread::spawn(move || {
                let result = process.wait();

                let code = match result {
                    Ok(status) => {
                        if let Some(signal) = status.signal() {
                            //TODO: is there a library for SIG to String?
                            if signal == 15 {
                                String::from("SIGTERM")
                            } else {
                                format!("SIG:{signal}")
                            }
                        } else if let Some(code) = status.code() {
                            format!("{code}")
                        } else {
                            unreachable!()
                        }
                    }
                    Err(_) => String::from("?"),
                };

                println!("[{i}] {cmd} exited with code {code}");

                if kill_others {
                    let _ = tx.send(());
                }
            }),
        ));
    }

    if kill_others {
        let pids = handles.iter().map(|(id, _)| *id).collect::<Vec<_>>();
        std::thread::spawn(move || {
            if rx.recv().is_ok() {
                process_killer(pids);
            }
        });
    }

    for (_, handle) in handles {
        let _ = handle.join();
    }
}

fn process_killer(pids: Vec<u32>) {
    println!("--> Sending SIGTERM to other processes..");
    for pid in pids {
        sig_term(pid);
    }
}

//TODO: impl windows via `taskkill`
// see `child_process.execSync(`taskkill /f /t /pid ${child.pid}`);`
// from nodejs impl: https://github.com/nodejs/node/blob/cb20c5b9f46c64d28bf495814fec5fe8a89b663d/benchmark/child_process/child-process-read.js#L35
fn sig_term(pid: u32) {
    unsafe {
        nix::libc::kill(pid as i32, SIGTERM);
    }
}

fn spawn_child(idx: usize, cmd: &str) -> Child {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute process");

    let child_out = child.stdout.take().expect("Failed to open stdout");
    std::thread::spawn(move || {
        let stdout_reader = BufReader::new(child_out);
        let stdout_lines = stdout_reader.lines();

        let mut stdout = std::io::stdout();

        for line in stdout_lines.map_while(Result::ok) {
            print!("[{idx}] ");
            stdout.write_all(line.as_bytes()).unwrap();
            println!();
        }
    });

    child
}
