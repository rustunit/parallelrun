mod cmd_ex;

use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use clap::Parser;
use cmd_ex::CommandExt;
use crossbeam::channel::{bounded, Receiver};
use sysinfo::Signal;

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

fn main() -> anyhow::Result<()> {
    let args = Arguments::parse();

    spawn_cmds(&args.cmd, args.kill_others, register_signals()?)
}

fn register_signals() -> anyhow::Result<Option<Receiver<Signal>>> {
    #[cfg(unix)]
    {
        use signal_hook::{consts::*, iterator::Signals};

        let (tx, rx) = bounded::<Signal>(1);
        let mut signals = Signals::new([SIGTERM, SIGINT, SIGHUP, SIGQUIT])?;

        thread::spawn(move || {
            let tx = tx.clone();
            for sig in signals.forever() {
                if sig != SIGINT {
                    tx.send(match sig {
                        SIGINT => Signal::Interrupt,
                        SIGHUP => Signal::Hangup,
                        SIGKILL => Signal::Kill,
                        SIGQUIT => Signal::Quit,
                        _ => Signal::Term,
                    })
                    .expect("problem sending signal");
                }
            }
        });

        Ok(Some(rx))
    }

    #[cfg(not(unix))]
    Ok(None)
}

fn spawn_cmds(
    args: &[String],
    kill_others: bool,
    signal_receiver: Option<Receiver<Signal>>,
) -> anyhow::Result<()> {
    let processes = args
        .iter()
        .enumerate()
        .map(|(i, arg)| (i, arg.to_string(), spawn_child(i, arg)))
        .collect::<Vec<(usize, String, Child)>>();

    let mut handles: Vec<(u32, JoinHandle<()>)> = vec![];
    let (tx, rx) = bounded::<Signal>(1);

    for (i, cmd, mut process) in processes {
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
                    let _ = tx.send(Signal::Term);
                }
            }),
        ));
    }

    let process_ids = handles.iter().map(|(id, _)| *id).collect::<Vec<_>>();

    let signal_in_flight = Arc::new(AtomicBool::new(false));

    if let Some(signal_receiver) = signal_receiver {
        thread::spawn({
            let pids = process_ids.clone();
            let signal_in_flight = signal_in_flight.clone();
            move || {
                if let Ok(signal) = signal_receiver.recv() {
                    signal_in_flight.store(true, Ordering::Relaxed);
                    process_killer(pids, signal);
                }
            }
        });
    }

    if kill_others {
        thread::spawn({
            let signal_in_flight = signal_in_flight.clone();
            move || {
                if let Ok(signal) = rx.recv() {
                    if !signal_in_flight.load(Ordering::Relaxed) {
                        process_killer(process_ids, signal);
                    }
                }
            }
        });
    }

    for (_, handle) in handles {
        let _ = handle.join();
    }

    Ok(())
}

#[cfg(unix)]
fn status_to_string(status: std::process::ExitStatus) -> String {
    use std::os::unix::process::ExitStatusExt;

    if let Some(signal) = status.signal() {
        signal_as_string(signal).into()
    } else if let Some(code) = status.code() {
        format!("{code}")
    } else {
        unreachable!()
    }
}

#[cfg(unix)]
#[allow(non_snake_case)]
fn signal_as_string(sig: i32) -> &'static str {
    use signal_hook::consts::*;

    //TODO: is there a library for SIG to String?
    match sig {
        SIGHUP => "SIGHUP",
        SIGINT => "SIGINT",
        SIGQUIT => "SIGQUIT",
        SIGILL => "SIGILL",
        SIGABRT => "SIGABRT",
        SIGFPE => "SIGFPE",
        SIGKILL => "SIGKILL",
        SIGSEGV => "SIGSEGV",
        SIGPIPE => "SIGPIPE",
        SIGALRM => "SIGALRM",
        SIGTERM => "SIGTERM",
        _ => "Unknown Signal",
    }
}

#[cfg(not(unix))]
fn signal_as_string(sig: i32) -> &'static str {
    "unkonwn signal"
}

#[cfg(not(unix))]
fn status_to_string(status: std::process::ExitStatus) -> String {
    if let Some(code) = status.code() {
        format!("{code}")
    } else {
        unreachable!()
    }
}

fn process_killer(process_ids: Vec<u32>, sig: Signal) {
    println!("--> Sending {sig} Signal to other processes..");
    for pid in process_ids {
        sig_term(pid, sig);
    }
}

fn sig_term(pid: u32, sig: Signal) {
    use sysinfo::{Pid, System, SUPPORTED_SIGNALS};
    let pid = Pid::from_u32(pid);
    let s = System::new_all();

    if let Some(process) = s.process(pid) {
        if SUPPORTED_SIGNALS.contains(&sig) {
            process.kill_with(sig).expect("kill_with failed");
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
