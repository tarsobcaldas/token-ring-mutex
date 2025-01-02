use std::net::UdpSocket;
use std::fmt::{self, Display, Formatter};
use std::error::Error;
use crate::peer::Request;

#[derive(Debug)]
pub struct ServerError {
    message: String,
}

impl ServerError {
    pub fn new(message: &str) -> ServerError {
        ServerError {
            message: message.to_string(),
        }
    }

    pub fn boxed(message: &str) -> Box<dyn Error> {
        Box::new(ServerError::new(message))
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ServerError {}

#[derive(Debug)]
pub struct Server {
    socket: UdpSocket,
}

impl Server {
    pub fn new(port: String) -> Result<Server, Box::<dyn Error>> {
        let address = format!("127.0.0.1:{}", port);
        let socket = UdpSocket::bind(address)?;
        Ok(Server { socket })
    }

    pub fn receive_requests(&self) -> Result<(Vec<Request>, String), Box::<dyn Error>> {
        let mut buffer = [0; 1024];
        let (length, src) = self.socket.recv_from(&mut buffer)?;
        let requests = String::from_utf8(buffer[..length].to_vec())?;
        let deserialized_requests: Vec<Request> = serde_json::from_str(&requests)?;
        Ok((deserialized_requests, src.to_string()))
    }

    pub fn process_request(&self, request: Request) -> String {
        let operation = request.operation.as_str();
        let arg1: i64 = request.arg1 as i64;
        let arg2: i64 = request.arg2 as i64;

        match operation {
            "add" => (arg1 + arg2).to_string(),
            "sub" => (arg1 - arg2).to_string(),
            "mul" => (arg1 * arg2).to_string(),
            "div" => {
                if arg2 == 0 {
                    return "Division by zero".to_string();
                }
                (arg1 as f64 / arg2 as f64).to_string()
            },
            _ => panic!("Invalid operation"),
        }
    }

    pub fn run(&self) -> Result<(), Box::<dyn Error>> {
        loop {
            let (requests, source) = self.receive_requests()?;
            println!("Processing request from {}", source);
            let queue = requests.clone();
            let answers: Vec<String> = queue.iter().map(|request| self.process_request(request.clone())).collect();
            for (request, response) in requests.iter().zip(answers.iter()) {
                println!("{} = {}", request, response);
            }
            println!();
            let serialized_response = serde_json::to_string(&answers)?;
            self.socket.send_to(serialized_response.as_bytes(), source)?;
        }
    }
}
