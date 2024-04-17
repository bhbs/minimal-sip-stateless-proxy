use std::collections::HashMap;
use std::net::UdpSocket;

use rsip::message::HeadersExt;
use rsip::HostWithPort;
use rsip::Request;
use rsip::Response;
use rsip::StatusCode;
use rsip::Uri;

use crate::error::Error;

pub struct Proxy {
    socket: UdpSocket,
    locations: HashMap<String, Uri>,
}

impl Proxy {
    pub fn new(host_with_port: HostWithPort) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(host_with_port.to_string())?;

        Ok(Self {
            socket,
            locations: HashMap::new(),
        })
    }

    pub fn run(&mut self) {
        let mut message_counter = 0;
        let mut buffer = [0; 0xffff];

        while let Ok((number_of_bytes, src_addr)) = self.socket.recv_from(&mut buffer) {
            message_counter += 1;
            println!("📥 message no.{} from: {}\n", message_counter, src_addr);

            if let Err(e) = self.process_message(&buffer[..number_of_bytes]) {
                eprintln!("{}\n", e);
            }
        }
    }

    fn process_message(&mut self, message: &[u8]) -> Result<(), Error> {
        if let Ok(request) = Request::try_from(message) {
            println!("{}", request.to_string());
            self.process_request(request)
        } else if let Ok(response) = Response::try_from(message) {
            println!("{}", response.to_string());
            self.process_response(response)
        } else {
            Err(Error::from(rsip::Error::Unexpected(
                "Failed to parse message.".to_string(),
            )))
        }
    }

    fn process_request(&mut self, request: Request) -> Result<(), Error> {
        let headers: rsip::Headers = request.headers.clone();

        let uri = request.to_header()?.uri()?;
        let user = uri
            .user()
            .ok_or(rsip::Error::Unexpected("Failed to get user.".to_string()))?;

        if request.method.to_string() == "REGISTER" {
            self.register(&request, user)?;

            let response = generate_response(request, 200);
            self.return_to_origin(response)
        } else {
            match self.locations.get(user) {
                Some(uri) => {
                    let request = rsip::Request {
                        method: request.method,
                        uri: uri.clone(),
                        headers,
                        version: Default::default(),
                        body: request.body,
                    };

                    self.send_message(uri.host_with_port.clone(), request.into())
                }
                None => {
                    let response = generate_response(request, 404);
                    self.return_to_origin(response)
                }
            }
        }
    }

    fn process_response(&self, response: Response) -> Result<(), Error> {
        self.return_to_origin(response)
    }

    fn return_to_origin(&self, response: Response) -> Result<(), Error> {
        let via = response.via_header()?;
        let host_with_port = via.uri()?.host_with_port;

        self.send_message(host_with_port, response.into())
    }

    fn send_message(&self, host_with_port: HostWithPort, message: String) -> Result<(), Error> {
        println!("🚀 message to: {}\n", host_with_port);
        println!("{}", message);

        self.socket
            .send_to(message.as_bytes(), host_with_port.to_string())?;

        Ok(())
    }

    fn register(&mut self, request: &Request, user: &str) -> Result<(), Error> {
        let contact = request.contact_header()?;
        let uri = contact.uri()?;

        println!("📝 register {}\n", user);
        println!("{}\n", uri);

        self.locations.insert(user.into(), uri);

        Ok(())
    }
}

fn generate_response(request: Request, status_code: u16) -> Response {
    Response {
        status_code: StatusCode::from(status_code),
        headers: request.headers,
        version: Default::default(),
        body: Default::default(),
    }
}
