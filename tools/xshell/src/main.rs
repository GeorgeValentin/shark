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
    // Connect to the local SSH server
    let tcp = TcpStream::connect("10.22.94.247:57522").unwrap();
    let mut sess = Session::new().unwrap();
    sess.handshake(&tcp).unwrap();
    sess.userauth_password("root", "n743exM87AqruHRu").unwrap();

    let mut channel = sess.channel_session().unwrap();

    eprintln!("requesting pty");

    channel.request_pty("xterm", None, None).unwrap();

    eprintln!("shell");

    channel.shell().unwrap();


    let (s1, r1) = crossbeam_channel::unbounded::<String>();
    let (s2, r2) = crossbeam_channel::unbounded::<String>();

    thread::spawn(move || loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                println!("{} bytes read", n);
                println!("{}", input);

                s1.send(input).unwrap();
            }
            Err(error) => println!("error: {}", error),
        }
    });

    loop {
        select! {
            recv(r1) -> line1  =>{
                println!("11111111111");
                if let Ok(line) = line1{
                    println!("line1: {}", line);

                    let cmd_string = line;
                    channel.write(cmd_string.as_bytes()).unwrap();

                    println!("11111111112");


                    if channel.exit_status().unwrap() > 0{

                        let mut stderr = vec![0; 4096];
                        channel.stderr().read(&mut stderr).unwrap();
                        println!("stderr: {}", String::from_utf8(stderr).unwrap());

                    }

                    println!("111111111113");

                    let mut stdout = vec![0; 4096];
                    channel.read(&mut stdout).unwrap();

                    println!("111111111114");

                    println!("stdout: {}", String::from_utf8(stdout).unwrap());

                }
            }
            recv(r2) -> line2 => {
                if let Ok(line) = line2{
                    println!("line2: {}", line);
                }
            }
//            default(Duration::from_millis(1000)) => {
//
//            }

        }
    }
}
