extern crate crossbeam;
extern crate tempfile;

use std::fs::File;
use std::io::prelude::*;
use std::net::TcpStream as StdTcpStream;
use std::process::Command;
use std::thread;

use tempfile::Builder;

use crossbeam::channel::{unbounded, TryRecvError};
use ssh2::Session;

fn main_ssh2() {
    let tcp = StdTcpStream::connect("192.168.33.30:22").unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.userauth_password("root", "vagrant").unwrap();

    let mut channel = sess.channel_session().unwrap();

    channel.request_pty("xterm", None, None).unwrap();

    channel.shell().unwrap();

    sess.set_blocking(false);

    let (trx, rev) = unbounded();

    thread::spawn(move || loop {
        let stdin = std::io::stdin();
        let mut line = String::new();
        stdin.read_line(&mut line).unwrap();
        trx.send(line).unwrap();
    });

    loop {
        let mut buf = vec![0; 4096];
        match channel.read(&mut buf) {
            Ok(_) => {
                let s = String::from_utf8(buf).unwrap();
                println!("{}", s);
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::WouldBlock {
                    println!("{}", e);
                }
            }
        }

        if !rev.is_empty() {
            match rev.try_recv() {
                Ok(line) => {
                    let cmd_string = line + "\n";
                    channel.write(cmd_string.as_bytes()).unwrap();
                    channel.flush().unwrap();
                }

                Err(TryRecvError::Empty) => {
                    println!("{}", "empty");
                }

                Err(TryRecvError::Disconnected) => {
                    println!("{}", "disconnected");
                }
            }
        }
    }
}

fn main() {
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
