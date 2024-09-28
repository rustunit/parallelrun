mod cmd_ex;

use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    thread::JoinHandle,
};

use clap::Parser;
use cmd_ex::CommandExt;
use crossbeam::channel::bounded;

#[derive(Default, Parser, Debug)]
#[command(version)]
struct Arguments {
    #[arg(
        default_value_t = false,
        short,
        long,
        help = "Kill other processes if one exits or dies. [bool]"
    )]
    kill_others: bool,

    #[arg(
        value_name = "COMMANDS",
        help = "List of commands to run. String escaped and space separated. ['cmd1' 'cmd2'...]"
    )]
    cmd: Vec<String>,
}

fn main() {
    let args = Arguments::parse();

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
                    Ok(status) => status_to_string(status),
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

#[cfg(unix)]
fn status_to_string(status: std::process::ExitStatus) -> String {
    use std::os::unix::process::ExitStatusExt;

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

#[cfg(not(unix))]
fn status_to_string(status: std::process::ExitStatus) -> String {
    if let Some(code) = status.code() {
        format!("{code}")
    } else {
        unreachable!()
    }
}

fn process_killer(pids: Vec<u32>) {
    println!("--> Sending SIGTERM to other processes..");
    for pid in pids {
        sig_term(pid);
    }
}

fn sig_term(pid: u32) {
    use sysinfo::{Pid, Signal, System, SUPPORTED_SIGNALS};
    let pid = Pid::from_u32(pid);
    let s = System::new_all();

    if let Some(process) = s.process(pid) {
        if SUPPORTED_SIGNALS.contains(&Signal::Term) {
            process.kill_with(Signal::Term).expect("kill_with failed");
        } else {
            process.kill();
        }
    }
}

fn spawn_child(idx: usize, cmd: &str) -> Child {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .with_no_window()
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn child process");

    let child_out = child.stdout.take().expect("Failed to open stdout");
    std::thread::spawn(move || {
        let stdout_reader = BufReader::new(child_out);
        let stdout_lines = stdout_reader.lines();

        let mut stdout = std::io::stdout();

        for line in stdout_lines.map_while(Result::ok) {
            print!("[{idx}] ");
            stdout
                .write_all(line.as_bytes())
                .expect("Failed to write to stdout");
            println!();
        }
    });

    child
}
