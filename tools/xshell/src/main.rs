#[macro_use]
extern crate crossbeam;

use crossbeam::crossbeam_channel;
use ssh2::Channel;
use ssh2::Session;
use std::io;
use std::io::prelude::*;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    let tcp = TcpStream::connect("192.168.33.20:22").unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.userauth_password("root", "vagrant").unwrap();

    let mut channel = sess.channel_session().unwrap();

    channel.request_pty("xterm", None, None).unwrap();

    channel.shell().unwrap();

    let (s1, r1) = crossbeam_channel::unbounded::<String>();
    let (s2, r2) = crossbeam_channel::unbounded::<String>();

    thread::spawn(move || loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                s1.send(input).unwrap();
            }
            Err(error) => println!("error: {}", error),
        }
    });

    let mut stdout = vec![0; 4096];
    channel.read(&mut stdout).unwrap();
    println!("{}", String::from_utf8(stdout).unwrap());

    let mut stdout = vec![0; 4096];
    channel.read(&mut stdout).unwrap();
    println!("{}", String::from_utf8(stdout).unwrap());

    loop {
        select! {
                            recv(r1) -> line1  =>{
                                if let Ok(line) = line1{

                                    let len = line.len();

                                    let cmd_string = line;
                                    channel.write(cmd_string.as_bytes()).unwrap();

                                    if channel.exit_status().unwrap() > 0{

                                        let mut stderr = vec![0; 4096];
                                        channel.stderr().read(&mut stderr).unwrap();
                                        println!("stderr: {}", String::from_utf8(stderr).unwrap());

                                    }

                                    let mut stdout = vec![0; 4096];
                                    channel.read(&mut stdout).unwrap();
                                    println!(">>{}", String::from_utf8(stdout).unwrap());

                                    if (len>1){
                                        let mut stdout = vec![0; 4096];
                                        channel.read(&mut stdout).unwrap();
                                        println!(">>{}", String::from_utf8(stdout).unwrap());
                                    }

                                }
                            }
                            recv(r2) -> line2 => {
                                if let Ok(line) = line2{
                                    println!("{}", line);
                                }
                            }
                            default(Duration::from_millis(1000)) => {

                            }

                        }
    }
}
