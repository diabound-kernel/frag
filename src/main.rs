mod thread_pool;

use {
    crate::thread_pool::ThreadPool,
    std::{
        fs,
        io::{prelude::*, BufReader},
        net::{TcpListener, TcpStream},
    },
};

fn handle_connection(mut stream: TcpStream) {
    let reader = BufReader::new(&mut stream);
    let request_line = reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", "static/index.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "static/404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    let thread_pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        thread_pool.execute(|| {
            handle_connection(stream.unwrap());
        });
    }
}
