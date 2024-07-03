use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct Message {
    sender: String,
    content: String,
}

fn handle_client(mut stream: TcpStream, id: usize, tx: Sender<Message>, rx: Receiver<Message>) {
    let mut buffer = [0; 1];
    let mut message = String::new();
    
    let welcome = format!("Добро пожаловать! Ваш ID: {}\n", id);
    stream.write_all(welcome.as_bytes()).unwrap();

    let receive_stream = stream.try_clone().unwrap();
    let mut receive_stream = receive_stream;

    // Поток для отправки сообщений клиенту
    thread::spawn(move || {
        loop {
            if let Ok(msg) = rx.recv() {
                let formatted_msg = format!("{}: {}\n", msg.sender, msg.content);
                if receive_stream.write(formatted_msg.as_bytes()).is_err() {
                    break;
                }
            }
        }
    });

    // Основной поток для чтения сообщений от клиента
    loop {
        match stream.read(&mut buffer) {
            Ok(size) => {
                if size == 0 { return; }
                let received_char = buffer[0] as char;
                if received_char == '\n' {
                    if !message.is_empty() {
                        tx.send(Message {
                            sender: id.to_string(),
                            content: message.clone(),
                        }).unwrap();
                        message.clear();
                    }
                } else {
                    message.push(received_char);
                }
            },
            Err(_) => return,
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Сервер запущен на 127.0.0.1:7878");

    let (tx, rx) = channel::<Message>();
    let client_channels: Arc<Mutex<HashMap<usize, Sender<Message>>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut client_id = 0;

    let broadcast_tx = tx.clone();
    let broadcast_channels = Arc::clone(&client_channels);
    thread::spawn(move || {
        loop {
            if let Ok(msg) = rx.recv() {
                let channels = broadcast_channels.lock().unwrap();
                for (id, client_tx) in channels.iter() {
                    if id.to_string() != msg.sender {
                        client_tx.send(msg.clone()).unwrap();
                    }
                }
            }
        }
    });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let (client_tx, client_rx) = channel::<Message>();
                let mut channels = client_channels.lock().unwrap();
                channels.insert(client_id, client_tx);
                drop(channels);

                let tx_clone = broadcast_tx.clone();
                
                thread::spawn(move || handle_client(stream, client_id, tx_clone, client_rx));
                
                client_id += 1;
            }
            Err(e) => {
                println!("Ошибка при подключении: {}", e);
            }
        }
    }
}