use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::io::{Write, stdin, stdout};
use std::fmt::{self, Display, Formatter};
use std::error::Error;
use rand::{prelude::*, thread_rng, Rng};
use statrs::distribution::{Poisson, PoissonError};
use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub struct PeerError {
    message: String,
}

impl PeerError {
    pub fn new(message: &str) -> PeerError {
        PeerError {
            message: message.to_string(),
        }
    }

    pub fn boxed(message: &str) -> Box<dyn Error> {
        Box::new(PeerError::new(message))
    }
}

impl Display for PeerError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for PeerError {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request {
    pub operation: String,
    pub arg1: i32,
    pub arg2: i32,
}

impl Display for Request {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.operation.as_str() {
            "add" => write!(f, "{} + {}", self.arg1, self.arg2),
            "sub" => write!(f, "{} - {}", self.arg1, self.arg2),
            "mul" => write!(f, "{} * {}", self.arg1, self.arg2),
            "div" => write!(f, "{} / {}", self.arg1, self.arg2),
            _ => write!(f, "Invalid operation"),
        }
    }
}


#[derive(Debug)]
pub struct Peer {
    pub address: String,
    pub socket: Arc<UdpSocket>,
    pub next_peer_address: String,
    pub server_address: String,
}

impl Peer {
    pub fn new(
        port: &str,
        next_peer_address: &str,
        server_address: &str,
    ) -> Result<Peer, Box::<dyn Error>> {
        let address = format!("127.0.0.1:{}", port);
        let socket = Arc::new(UdpSocket::bind(address.clone())?);
        let next_peer_address = next_peer_address.to_string();
        let server_address = server_address.to_string();
        Ok(Peer {
            address,
            socket,
            next_peer_address,
            server_address,
        })
    }

    pub fn generate_request() -> Request {
        let mut rng = thread_rng();
        let operation = ["add", "sub", "mul", "div"].choose(&mut rng).unwrap();
        let arg1: i32 = rng.gen();
        let arg2: i32 = rng.gen();

        Request {
            operation: operation.to_string(),
            arg1,
            arg2,
        }
    }

    pub fn send_request(
        socket: &UdpSocket,
        request: &Vec<Request>,
        server_address: &str,
    ) -> Result<(), Box<dyn Error>> {
        let serialized_request = serde_json::to_string(&request)?;
        println!("Sending {} requests to server", request.len());
        socket.send_to(serialized_request.as_bytes(), server_address)?;
        let mut buffer = [0; 1024];
        let (length, _) = socket.recv_from(&mut buffer)?;
        let answer = String::from_utf8_lossy(&buffer[..length]).to_string();
        let deserialized_response: Vec<String> = serde_json::from_str(&answer)?;
        for (request, response) in request.iter().zip(deserialized_response.iter()) {
            println!("{} = {}", request, response);
        }
        println!();
        Ok(())
    }

    pub fn send_message(socket: &UdpSocket, message: &str, peer_address: &str) -> Result<(), Box::<dyn Error>> {
        socket.send_to(message.as_bytes(), peer_address)?;
        Ok(())
    }

    pub fn receive_response(socket: &UdpSocket) -> Result<String, std::io::Error> {
        let mut buffer = [0; 1024];
        let (length, _) = socket.recv_from(&mut buffer)?;
        Ok(String::from_utf8_lossy(&buffer[..length]).to_string())
    }

    pub fn send_token(
        socket: &UdpSocket,
        next_peer_address: &str,
    ) -> Result<(), std::io::Error> {
        println!("Sending token to {}", next_peer_address);
        socket.send_to("token".as_bytes(), next_peer_address)?;
        println!("Token sent");
        Ok(())
    }

    pub fn check_peer(
        socket: &UdpSocket,
        next_peer_address: &str,
    ) -> Result<bool, Box::<dyn Error>> {
        socket.send_to("check".as_bytes(), next_peer_address)?;
        let mut buffer = [0; 1024];
        match socket.recv_from(&mut buffer) {
            Ok((length, _)) => {
                let response = String::from_utf8_lossy(&buffer[..length]).to_string();
                Ok(response == "ok")
            }
            Err(_) => Ok(false),
        }
    }

    pub fn handle_token(
        socket: &UdpSocket,
        queue: &Arc<Mutex<Vec<Request>>>,
        next_peer_address: &str,
        server_address: &str,
    ) -> Result<(), Box::<dyn Error>>{
        let mut queue = queue.lock().unwrap();
        if !queue.is_empty() {
            Peer::send_request(socket, &queue, server_address)?;
            queue.clear();
        }
        Peer::send_token(socket, next_peer_address)?;
        Ok(())
    }

    pub fn listen(&mut self) -> Result<(), Box::<dyn Error>> {
        let next_peer_address = self.next_peer_address.clone();
        let server_address = self.server_address.clone();
        let queue = Arc::new(Mutex::new(Vec::<Request>::new()));
        let queue_clone = Arc::clone(&queue);
        let socket_clone = Arc::clone(&self.socket);
        thread::spawn(move || {
            loop {
                let wait_time = poisson_event_rate(4.0).unwrap();
                let mut queue = queue_clone.lock().unwrap();
                let request = Peer::generate_request();
                queue.push(request);
                thread::sleep(wait_time);
            }
        });

        thread::spawn(move || {
            loop {
                println!("Listening for messages");
                let socket = socket_clone.as_ref();
                let response = Peer::receive_response(socket).unwrap();
                let _ = match response.as_str() {
                    "token" => {
                        println!("Received token");
                        Peer::handle_token(socket, &queue, &next_peer_address, &server_address)
                    },
                    "check" => Peer::send_message(socket, "ok", &next_peer_address),
                    _ => Err(PeerError::boxed("Invalid response")),
                };
            }
        });

        loop {
            thread::park();
        }
    }

    pub fn start(&mut self) -> Result<(), Box::<dyn Error>> {
        let next_peer_address = self.next_peer_address.clone();
        Peer::send_token(self.socket.as_ref(), &next_peer_address)?;
        self.listen()
    }

    pub fn run(&mut self) -> Result<(), Box::<dyn Error>>{
        loop {
            print!("> ");
            stdout().flush().unwrap();
            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();
            match input.trim() {
                "listen" => return self.listen(),
                "start" => return self.start(),
                "check" => {
                    let result = Peer::check_peer(self.socket.as_ref(), &self.next_peer_address);
                    match result {
                        Ok(true) => {
                            println!("Peer is alive");
                            continue
                        },
                        Ok(false) => {
                            println!("Peer is dead");
                            continue
                        },
                        Err(_) =>  {
                            eprintln!("Error checking peer");
                            continue
                        },
                    }
                }
                "exit" => break,
                _ => println!("Invalid command"),
            }
        }
        Ok(())
    }
}

pub fn poisson_event_rate(lambda: f64) -> Result<Duration, PoissonError> {
    let poisson = Poisson::new(60.0/lambda)?;
    let mut rng = thread_rng();
    let wait_time = poisson.sample(&mut rng);
    Ok(Duration::from_secs_f64(wait_time))
}
