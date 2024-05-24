use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(Debug)]
pub struct HttpResponse {
    status_code: u16,
    headers: HashMap<String, String>,
    body: String,
}

impl HttpResponse {
    pub fn new(status_code: u16, body: String) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "text/html".to_string());
        headers.insert("Content-Length".to_string(), body.len().to_string());
        HttpResponse {
            status_code,
            headers,
            body,
        }
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    pub fn to_string(&self) -> String {
        let headers: String = self
            .headers
            .iter()
            .map(|(k, v)| format!("{}: {}\r\n", k, v))
            .collect();
        format!(
            "HTTP/1.1 {} OK\r\n{}Content-Length: {}\r\n\r\n{}",
            self.status_code,
            headers,
            self.body.len(),
            self.body
        )
    }
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    method: String,
    path: String,
    http_version: String,
    headers: HashMap<String, String>,
}

impl HttpRequest {
    pub fn new(
        method: &str,
        path: &str,
        http_version: &str,
        headers: HashMap<String, String>,
    ) -> Self {
        HttpRequest {
            method: method.to_string(),
            path: path.to_string(),
            http_version: http_version.to_string(),
            headers,
        }
    }

    fn from_buffer(buffer: &[u8]) -> Result<Self, &'static str> {
        let request = String::from_utf8_lossy(buffer);
        let mut lines = request.lines();

        let request_line = lines.next().ok_or("Request line is missing")?;
        let mut parts = request_line.split_whitespace();
        let method = parts.next().ok_or("Method is missing")?.to_string();
        let path = parts.next().ok_or("Path is missing")?.to_string();
        let http_version = parts.next().ok_or("HTTP version is missing")?.to_string();

        let mut headers = HashMap::new();
        for line in lines {
            if line.is_empty() {
                break;
            }
            let mut header_parts = line.splitn(2, ":");
            let key = header_parts
                .next()
                .ok_or("Header key is missing")?
                .trim()
                .to_string();
            let value = header_parts
                .next()
                .ok_or("Header value is missing")?
                .trim()
                .to_string();
            headers.insert(key, value);
        }
        let parsed_request = HttpRequest::new(
            method.as_str(),
            path.as_str(),
            http_version.as_str(),
            headers,
        );
        println!("!!!! {:?}", parsed_request);
        Ok(parsed_request)
    }
}

struct RequestHandler {}

impl RequestHandler {
    pub fn handle(request_buffer: &[u8], routes: &[Route]) -> Option<String> {
        let request_parsed = HttpRequest::from_buffer(request_buffer).ok()?;
        let mut path = request_parsed.clone().path;
        println!("path:{}", path);
        if path.starts_with("/") {
            path.remove(0);
        }
        for route in routes.iter() {
            let response =
                HttpResponse::new(200, "<body><h1>Good bye, Browser!</h1></body>".to_string());
            if route.path == path {
                let result = route.process_request(request_parsed.clone(), response);
                return Some(result.to_string());
            }
            break;
        }
        None
    }
}

#[derive(Clone)]
pub struct Route {
    pub(crate) path: &'static str,
}

impl Route {
    pub fn process_request(&self, req: HttpRequest, res: HttpResponse) -> HttpResponse {
        println!("request handled, {}", req.path);
        res
    }
}

pub struct Server {
    address: String,
    routes: Vec<Route>,
}

impl Server {
    pub fn new(ip: &str, port: u16) -> Self {
        let address = format!("{}:{}", ip, port);
        Server {
            address,
            routes: Vec::new(),
        }
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn routes(mut self, routes: &[Route]) -> Self {
        self.routes.extend_from_slice(routes);
        self
    }

    pub async fn listen(&self) -> std::io::Result<&str> {
        println!("Starting server on port {}", self.address);
        let listener = TcpListener::bind(self.address()).await?;
        let routes = self.routes.clone();
        loop {
            let (mut socket, _) = listener.accept().await?;
            let routes = routes.clone();
            tokio::spawn(async move {
                let mut buf = vec![0; 1024];
                loop {
                    match socket.read(&mut buf).await {
                        Ok(0) => return "OK",
                        Ok(n) => match RequestHandler::handle(&buf[..n], &routes) {
                            Some(response) => {
                                if let Err(e) = socket.write_all(&response.as_bytes()).await {
                                    eprintln!("Failed to write to socket; err = {:?}", e);
                                    return "There is an error occurred";
                                }
                            }
                            None => {
                                let error_html = "<body><h1>Wrong!</h1></body>".to_string();
                                let error_response = HttpResponse::new(404, error_html).to_string();
                                let error_response_str = error_response.as_str();

                                if let Err(e) =
                                    socket.write_all(error_response_str.as_bytes()).await
                                {
                                    eprintln!(
                                        "Failed to write error response to socket; err = {:?}",
                                        e
                                    );
                                    return "There is an error occurred";
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to read from socket; err = {:?}", e);
                            return "There is an error occurred";
                        }
                    }
                }
            });
        }
    }
}
