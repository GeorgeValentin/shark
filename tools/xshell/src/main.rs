extern crate crossbeam;
extern crate rustyline;

use ssh2::Session;
use std::io::prelude::*;
use std::net::TcpStream;
use std::thread;
use std::time;

use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() {
    let tcp = TcpStream::connect("192.168.33.20:22").unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.userauth_password("root", "vagrant").unwrap();

    let mut channel = sess.channel_session().unwrap();

    channel.request_pty("xterm", None, None).unwrap();

    channel.shell().unwrap();

    let mut stdout = vec![0; 4096];
    channel.read(&mut stdout).unwrap();
    let s = String::from_utf8(stdout).unwrap();
    println!("{}", s);

    let mut stdout = vec![0; 4096];
    channel.read(&mut stdout).unwrap();
    let s = String::from_utf8(stdout).unwrap();
    println!("{}", s);

    let mut reader = Editor::<()>::new();

    loop {
        let readline = reader.readline(">>");

        match readline {
            Ok(line) => {
                let len = line.len();
                let cmd_string = line + "\n";
                channel.write(cmd_string.as_bytes()).unwrap();
                channel.flush().unwrap();

                thread::sleep(time::Duration::from_secs(1));

                let mut stdout = vec![0; 4096];
                channel.read(&mut stdout).unwrap();
                let s = String::from_utf8(stdout).unwrap();
                println!("{}", s);

                //                if len > 0 {
                //                    let mut stdout = vec![0; 4096];
                //                    channel.read(&mut stdout).unwrap();
                //                    let s = String::from_utf8(stdout).unwrap();
                //                    println!("{}", s);
                //                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }
    }
}
