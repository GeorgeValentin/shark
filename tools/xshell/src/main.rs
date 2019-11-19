extern crate crossbeam;

use crossbeam::channel::{unbounded, TryRecvError};
use ssh2::Session;
use std::io::prelude::*;
use std::net::TcpStream as StdTcpStream;
use std::thread;

fn main() {
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
