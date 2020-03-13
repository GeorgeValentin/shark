//#[macro_use]
//extern crate prettytable;

use std::fs;
use std::fs::File;
use std::io::Read;
use std::ptr::null;

use prettytable::{Table, Row, Cell, Attr, color};

#[derive(Debug)]
pub struct Host {
    pub hostname: String,
    pub ip: String,
    pub port: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct Group {
    pub name: String,
    pub host_list: Vec<Host>,
}

#[derive(Debug)]
pub struct Config {
    pub group_list: Vec<Group>,
}

impl Config {
    pub fn pretty_print(self: Self) {
        for group in self.group_list {
            let mut table = Table::new();
            let mut table2 = Table::new();
//
//            table.add_row(table2.into());

            table.add_row(Row::new(vec![
                Cell::new("NO").with_style(Attr::ForegroundColor(color::GREEN)),
                Cell::new("hostname"),
                Cell::new("ip"),
                Cell::new("port"),
                Cell::new("username"),
                Cell::new("password")
            ]));

            for (i, host) in group.host_list.iter().enumerate() {
                table.add_row(Row::new(vec![
                    Cell::new(format!("{}", i + 1).as_str()).with_style(Attr::ForegroundColor(color::RED)),
                    Cell::new(host.hostname.as_str()),
                    Cell::new(host.ip.as_str()),
                    Cell::new(host.port.as_str()),
                    Cell::new(host.username.as_str()),
                    Cell::new(host.password.as_str())
                ]));
            }

            table2.add_row(Row::new(vec![
                Cell::new(format!("Grop Name: {}", group.name).as_str()).with_style(Attr::ForegroundColor(color::GREEN)),
            ]));
            table2.add_row(Row::new(vec![
                Cell::new(table.to_string().as_str())
            ]));
            table2.printstd();
        }
    }
}


pub fn parse_group(group_name: &str, file_path: &str) -> Option<Group> {
    let mut file = File::open(file_path).unwrap();
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).unwrap();
    println!("{:?}", buffer);
    drop(file);

    let lines: Vec<&str> = buffer.split("\n").collect();
    if lines.len() == 0 {
        return None;
    }

    let mut group = Group {
        name: group_name.to_string(),
        host_list: vec![],
    };

    for line in lines {
        if line.len() == 0 {
            continue;
        }
        if line.starts_with("#") {
            continue;
        }
        println!("{}", line);
        let span_list: Vec<&str> = line.split_whitespace().collect();
        if span_list.len() < 5 {
            continue;
        }

        let hostname = span_list[0];
        let ip = span_list[1];
        let port = span_list[2];
        let username = span_list[3];
        let password = span_list[4];

        let host = Host {
            hostname: hostname.to_string(),
            ip: ip.to_string(),
            port: port.to_string(),
            username: username.to_string(),
            password: password.to_string(),
        };

        group.host_list.push(host);
    }

    return Some(group);
}

pub fn parse_config(config_dir: &str) -> Config {
    let readDir = fs::read_dir(config_dir).unwrap();

    let mut config = Config {
        group_list: vec![]
    };

    for e in readDir {
        println!("{:?}", e);
        let entry = e.unwrap();
        let file_name = entry.file_name();
        let file_name = file_name.to_str().unwrap().to_string();
        let arr: Vec<&str> = file_name.split(".").collect();
        let file_name = arr[0];
        let group = parse_group(file_name, entry.path().to_str().unwrap());

        println!("{:?}", group);


        match group {
            Some(group) => {
                config.group_list.push(group);
            }
            None => {}
        }
    }


    return config;
}