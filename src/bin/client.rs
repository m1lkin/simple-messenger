use std::net::TcpStream;
use std::io::{Read, Write, stdin};
use std::thread;

fn main() {
    match TcpStream::connect("localhost:7878") {
        Ok(mut stream) => {
            println!("Успешно подключено к серверу на 127.0.0.1:7878");

            let mut receive_stream = stream.try_clone().unwrap();

            // Поток для приема сообщений
            thread::spawn(move || {
                let mut buffer = [0; 1];
                while let Ok(size) = receive_stream.read(&mut buffer) {
                    if size == 0 { break; }
                    print!("{}", buffer[0] as char);
                }
            });

            // Основной поток для отправки сообщений
            loop {
                let mut input = String::new();
                stdin().read_line(&mut input).unwrap();
                stream.write_all(input.as_bytes()).unwrap();
            }
        },
        Err(e) => {
            println!("Не удалось подключиться: {}", e);
        }
    }
}