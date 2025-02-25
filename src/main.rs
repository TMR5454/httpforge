use std::env;
//use std::fmt::format;
use anyhow::{Context, Result};
use env_logger;
use log;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpListener;

fn handle_connection(mut stream: std::net::TcpStream) -> Result<()> {
    let mut buffer = [0; 1024];

    /* read request */
    let _n = stream.read(&mut buffer)?;
    let s = String::from_utf8(buffer.to_vec()).context("parse Error")?;
    log::debug!("\n{}", s);

    /* request parse */
    match s.split_whitespace().collect::<Vec<&str>>().as_slice() {
        ["GET", path, more @ ..] => {
            log::debug!("GET {}", path);

            let path = if *path == "/" {
                std::env::current_dir().context("current_dir()")?.join("index.html")
            } else {
                let fullpath = std::env::current_dir().context("current_dir()")?.to_str().context("to_str()")?.to_string();
                std::path::PathBuf::from(fullpath + path)
            };

            log::debug!("path: {:?}", path);
            log::debug!("more: {:?}", more);

            let file = File::open(path);
            let mut response = String::with_capacity(32);

            match file {
                Ok(mut file) => {
                    let mut body = String::new();
                    file.read_to_string(&mut body)
                        .context("fail to read file")?;

                    response = format!("HTTP/1.1 200 OK\r\n\r\n{}", body);
                }

                Err(e) => {
                    response = format!("HTTP/1.1 404 NOT FOUND\r\n\r\n{}{}", "404 NOT FOUND. ", e);
                }
            }

            let _n = stream.write(response.as_bytes()).context("fail to reply")?;
            stream.flush().context("fail to flush")?;
        }
        ["POST", ..] => {
            log::debug!("POST");
        }
        _ => {
            log::debug!("other");
            log::debug!(
                "{:?}",
                s.split_whitespace().collect::<Vec<&str>>().as_slice()
            );
        }
    }

    Ok(())
}

fn http_server() -> Result<()> {
    let linstener = TcpListener::bind("0.0.0.0:8080")?;

    for stream in linstener.incoming() {
        let stream = stream.context("faile to create tcp-stream")?;

        handle_connection(stream)?;
    }
    Ok(())
}

fn main() {
    env::set_var("RUST_BACKTRACE", "full");
    env::set_var("RUST_LOG", "trace");
    env_logger::init();

    let _ret = http_server();
}
