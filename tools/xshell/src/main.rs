extern crate clap;
extern crate crossbeam;
extern crate tempfile;
extern crate mio;

use std::fs::File;
use std::io::prelude::*;
use std::net::TcpStream as StdTcpStream;
use std::process::{Command, exit};
use std::io;

use tempfile::Builder;
use ssh2::Session;

use clap::{App, Arg, SubCommand};

use mio::net::{TcpListener as MioTcpListener, TcpStream as MioTcpStream};
use mio::{Events, Poll, Token, Interest};

mod xshell_config;

fn main2() {
    let version = env!("CARGO_PKG_VERSION");
    let authors = env!("CARGO_PKG_AUTHORS");
    let about = env!("CARGO_PKG_DESCRIPTION");

    let mut command_list = Vec::new();

    let cmd_list = SubCommand::with_name("list").about("list all the host in group");

    command_list.push(cmd_list);

    let mut app = App::new("xshell")
        .version(version)
        .author(authors)
        .about(about)
        .arg(
            Arg::with_name("config dir")
                .short("d")
                .long("config_dir")
                .value_name("DIR")
                .help("Sets a custom config dir")
                .takes_value(true),
        )
        .subcommands(command_list);

    let app_matches = app.clone().get_matches();

    let config_dir = app_matches
        .value_of("config_dir")
        .unwrap_or("etc/server_groups");
    println!("Value for config dir: {}", config_dir);

    match app_matches.subcommand() {
        ("list", Some(sub_m)) => {
            let config = xshell_config::parse_config(config_dir);
            println!("{:?}", config);
            config.pretty_print();
        }
        _ => {
            println!("unknown subcommand");
            app.print_help();
        }
    }

    return;

    let username = "root";
    let password = "vagrant";
    let ip = "192.168.33.30";
    let port = "22";
    let server_name = "vag";

    let mut file = File::open("./ssh.expect").unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).unwrap();
    drop(file);

    let buffer = buffer.replace("__username__", username);
    let buffer = buffer.replace("__password__", password);
    let buffer = buffer.replace("__ip__", ip);
    let buffer = buffer.replace("__port__", port);
    let buffer = buffer.replace("__server_name__", server_name);

    let tmp_dir = Builder::new().prefix("xshell_rs.").tempdir().unwrap();
    let file_path = tmp_dir.path().join("ssh.expect");

    let mut file = File::create(file_path.to_str().unwrap()).unwrap();
    file.write_all(buffer.as_bytes());
    file.sync_all();
    drop(file);

    let mut cmd = Command::new("expect");

    cmd.arg("-f").arg(file_path.to_str().unwrap());

    let mut process = cmd.spawn().expect("process failed");

    process.wait().unwrap();

    tmp_dir.close().unwrap();
}

const SSH_TOKEN: Token = Token(1);

fn main() {

    let addr = "192.168.33.30:22".parse().unwrap();

    let mut tcp = MioTcpStream::connect(addr).unwrap();


    let mut poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1024);
    poll.registry().register(&mut tcp, SSH_TOKEN, Interest::READABLE).unwrap();


    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.userauth_password("root", "vagrant").unwrap();

    let mut channel = sess.channel_session().unwrap();

    channel.request_pty("xterm", None, None).unwrap();

    channel.shell().unwrap();

    sess.set_blocking(false);

    let mut ssh_stdin = channel.stream(0);
    let mut ssh_stdout = channel.stream(0);
    let mut ssh_stderr = channel.stderr();

    let all_result: Result<u64, io::Error> = crossbeam::scope(|s| {
        let stdin_handle = s.spawn(|_| {
            let stdin = io::stdin();
            let mut stdin = stdin.lock();

            loop {
                let mut line = String::new();
                stdin.read_line(&mut line).unwrap();
                ssh_stdin.write(line.as_bytes()).unwrap();
                ssh_stdin.flush().unwrap();
            }
        });

        let poll_handle = s.spawn(|_| {
            loop {
                poll.poll(&mut events, None).unwrap();
                for event in events.iter() {
                    match event.token() {
                        SSH_TOKEN => {

                            if event.is_readable() {
                                let stdout = io::stdout();
                                let mut stdout = stdout.lock();
                                let mut buf = vec![0; 4096];
                                match ssh_stdout.read(&mut buf) {
                                    Ok(_) => {
                                        let s = String::from_utf8(buf).unwrap();
                                        stdout.write(s.as_bytes()).unwrap();
                                    }
                                    Err(e) => {
                                        if e.kind() != std::io::ErrorKind::WouldBlock {
                                            println!("{}", e);
                                        }
                                    }
                                }

                                let stderr = io::stderr();
                                let mut stderr = stderr.lock();
                                let mut buf = vec![0; 4096];
                                match ssh_stderr.read(&mut buf) {
                                    Ok(_) => {
                                        let s = String::from_utf8(buf).unwrap();
                                        stderr.write(s.as_bytes()).unwrap();
                                    }
                                    Err(e) => {
                                        if e.kind() != std::io::ErrorKind::WouldBlock {
                                            println!("{}", e);
                                        }
                                    }
                                }
                            }

                        }
                        _ => unreachable!(),
                    }
                }
            }
        });

        let stdin_result: Result<u64, io::Error> = stdin_handle.join().unwrap();
        let poll_result: Result<u64, io::Error> = poll_handle.join().unwrap();

        stdin_result.and(poll_result)

    }).unwrap(); // Should never panic because all scoped threads have been joined.

    all_result.unwrap();

    // Wait for SSH channel to close.
    channel.wait_close().unwrap();
    let exit_status = channel.exit_status().unwrap();
    exit(exit_status)
}
