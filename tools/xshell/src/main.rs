extern crate clap;
extern crate crossbeam;
extern crate tempfile;

use std::fs::File;
use std::io::prelude::*;
use std::net::TcpStream as StdTcpStream;
use std::process::Command;
use std::thread;
use std::{fs, io};

use tempfile::Builder;

use crossbeam::channel::{unbounded, TryRecvError};
use ssh2::Session;

use clap::{App, Arg, SubCommand};


mod xshell_config;

fn main() {
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
