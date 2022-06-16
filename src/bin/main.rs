use std::net::{TcpListener, TcpStream};
// We bring std::io::prelude into scope to get access to certain traits that let us read from and write to the stream.
use std::fs;
use std::io::prelude::*;
use std::thread;
use std::time::Duration;
use web_server::ThreadPool;

// It therefore needs to be mut because its internal state might change; usually, we think of “reading” as not needing mutation, but in this case we need the mut keyword.
fn handle_connection(mut stream: TcpStream) {
    // We’ve made the buffer 1024 bytes in size, which is big enough to hold the data of a basic request
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();
    // he “lossy” part of the name indicates the behavior of this function when it sees an invalid UTF-8 sequence: it will replace the invalid sequence with �, the U+FFFD REPLACEMENT CHARACTER.
    // println!("Request {}", String::from_utf8_lossy(&buffer[..]));

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "index.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "index.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents,
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

// A thread pool is a group of spawned threads that are waiting and ready to handle a task. When the program receives a new task, it assigns one of the threads in the pool to the task, and that thread will process the task. The remaining threads in the pool are available to handle any other tasks that come in while the first thread is processing. When the first thread is done processing its task, it’s returned to the pool of idle threads, ready to handle a new task. A thread pool allows you to process connections concurrently, increasing the throughput of your server.
fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}
