// HTTP client and server functionality for the Bulu programming language
// Requirements: 7.2.1, 7.2.3, 7.2.4

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

/// HTTP methods supported by the client and server
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

impl HttpMethod {
    pub fn from_str(method: &str) -> Option<HttpMethod> {
        match method.to_uppercase().as_str() {
            "GET" => Some(HttpMethod::GET),
            "POST" => Some(HttpMethod::POST),
            "PUT" => Some(HttpMethod::PUT),
            "DELETE" => Some(HttpMethod::DELETE),
            "PATCH" => Some(HttpMethod::PATCH),
            "HEAD" => Some(HttpMethod::HEAD),
            "OPTIONS" => Some(HttpMethod::OPTIONS),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
        }
    }
}

/// HTTP status codes
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum HttpStatus {
    Ok = 200,
    Created = 201,
    NoContent = 204,
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
}

impl HttpStatus {
    pub fn from_code(code: u16) -> Option<HttpStatus> {
        match code {
            200 => Some(HttpStatus::Ok),
            201 => Some(HttpStatus::Created),
            204 => Some(HttpStatus::NoContent),
            400 => Some(HttpStatus::BadRequest),
            401 => Some(HttpStatus::Unauthorized),
            403 => Some(HttpStatus::Forbidden),
            404 => Some(HttpStatus::NotFound),
            405 => Some(HttpStatus::MethodNotAllowed),
            500 => Some(HttpStatus::InternalServerError),
            501 => Some(HttpStatus::NotImplemented),
            502 => Some(HttpStatus::BadGateway),
            503 => Some(HttpStatus::ServiceUnavailable),
            _ => None,
        }
    }

    pub fn code(&self) -> u16 {
        *self as u16
    }

    pub fn reason_phrase(&self) -> &'static str {
        match self {
            HttpStatus::Ok => "OK",
            HttpStatus::Created => "Created",
            HttpStatus::NoContent => "No Content",
            HttpStatus::BadRequest => "Bad Request",
            HttpStatus::Unauthorized => "Unauthorized",
            HttpStatus::Forbidden => "Forbidden",
            HttpStatus::NotFound => "Not Found",
            HttpStatus::MethodNotAllowed => "Method Not Allowed",
            HttpStatus::InternalServerError => "Internal Server Error",
            HttpStatus::NotImplemented => "Not Implemented",
            HttpStatus::BadGateway => "Bad Gateway",
            HttpStatus::ServiceUnavailable => "Service Unavailable",
        }
    }
}

/// HTTP request structure
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    pub fn new(method: HttpMethod, path: String) -> Self {
        HttpRequest {
            method,
            path,
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        let body_len = body.len();
        self.body = body;
        self.headers.insert("Content-Length".to_string(), body_len.to_string());
        self
    }

    pub fn with_json_body(mut self, json: String) -> Self {
        self.body = json.into_bytes();
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        self.headers.insert("Content-Length".to_string(), self.body.len().to_string());
        self
    }

    pub fn get_header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }

    pub fn body_as_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }
}

/// HTTP response structure
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub version: String,
    pub status: HttpStatus,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn new(status: HttpStatus) -> Self {
        HttpResponse {
            version: "HTTP/1.1".to_string(),
            status,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        let body_len = body.len();
        self.body = body;
        self.headers.insert("Content-Length".to_string(), body_len.to_string());
        self
    }

    pub fn with_json_body(mut self, json: String) -> Self {
        self.body = json.into_bytes();
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        self.headers.insert("Content-Length".to_string(), self.body.len().to_string());
        self
    }

    pub fn with_text_body(mut self, text: String) -> Self {
        self.body = text.into_bytes();
        self.headers.insert("Content-Type".to_string(), "text/plain".to_string());
        self.headers.insert("Content-Length".to_string(), self.body.len().to_string());
        self
    }

    pub fn body_as_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut response = Vec::new();
        
        // Status line
        let status_line = format!("{} {} {}\r\n", 
            self.version, self.status.code(), self.status.reason_phrase());
        response.extend_from_slice(status_line.as_bytes());
        
        // Headers
        for (key, value) in &self.headers {
            let header_line = format!("{}: {}\r\n", key, value);
            response.extend_from_slice(header_line.as_bytes());
        }
        
        // Empty line
        response.extend_from_slice(b"\r\n");
        
        // Body
        response.extend_from_slice(&self.body);
        
        response
    }
}

/// HTTP client for making requests
pub struct HttpClient {
    default_headers: HashMap<String, String>,
}

impl HttpClient {
    pub fn new() -> Self {
        let mut default_headers = HashMap::new();
        default_headers.insert("User-Agent".to_string(), "Bulu-HTTP-Client/1.0".to_string());
        
        HttpClient {
            default_headers,
        }
    }

    pub fn with_default_header(mut self, key: String, value: String) -> Self {
        self.default_headers.insert(key, value);
        self
    }

    pub fn get(&self, url: &str) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let request = HttpRequest::new(HttpMethod::GET, url.to_string());
        self.send_request(request)
    }

    pub fn post(&self, url: &str, body: Vec<u8>) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let request = HttpRequest::new(HttpMethod::POST, url.to_string())
            .with_body(body);
        self.send_request(request)
    }

    pub fn put(&self, url: &str, body: Vec<u8>) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let request = HttpRequest::new(HttpMethod::PUT, url.to_string())
            .with_body(body);
        self.send_request(request)
    }

    pub fn delete(&self, url: &str) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let request = HttpRequest::new(HttpMethod::DELETE, url.to_string());
        self.send_request(request)
    }

    pub fn patch(&self, url: &str, body: Vec<u8>) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let request = HttpRequest::new(HttpMethod::PATCH, url.to_string())
            .with_body(body);
        self.send_request(request)
    }

    fn send_request(&self, mut request: HttpRequest) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        // Add default headers
        for (key, value) in &self.default_headers {
            if !request.headers.contains_key(key) {
                request.headers.insert(key.clone(), value.clone());
            }
        }

        // Parse URL (simplified - in real implementation would use proper URL parsing)
        let url_parts: Vec<&str> = request.path.split('/').collect();
        if url_parts.len() < 3 {
            return Err("Invalid URL format".into());
        }

        let host = url_parts[2];
        let path = if url_parts.len() > 3 {
            format!("/{}", url_parts[3..].join("/"))
        } else {
            "/".to_string()
        };

        // Connect to server
        let mut stream = TcpStream::connect(format!("{}:80", host))?;

        // Build HTTP request
        let mut http_request = format!("{} {} {}\r\n", 
            request.method.as_str(), path, request.version);
        
        // Add Host header
        request.headers.insert("Host".to_string(), host.to_string());
        
        // Add headers
        for (key, value) in &request.headers {
            http_request.push_str(&format!("{}: {}\r\n", key, value));
        }
        
        http_request.push_str("\r\n");
        
        // Send request
        stream.write_all(http_request.as_bytes())?;
        if !request.body.is_empty() {
            stream.write_all(&request.body)?;
        }

        // Read response
        let mut response_data = Vec::new();
        stream.read_to_end(&mut response_data)?;

        // Parse response (simplified)
        let response_str = String::from_utf8_lossy(&response_data);
        let lines: Vec<&str> = response_str.split("\r\n").collect();
        
        if lines.is_empty() {
            return Err("Empty response".into());
        }

        // Parse status line
        let status_parts: Vec<&str> = lines[0].split_whitespace().collect();
        if status_parts.len() < 3 {
            return Err("Invalid status line".into());
        }

        let status_code: u16 = status_parts[1].parse()?;
        let status = HttpStatus::from_code(status_code)
            .unwrap_or(HttpStatus::InternalServerError);

        // Parse headers
        let mut headers = HashMap::new();
        let mut body_start = 0;
        
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.is_empty() {
                body_start = i + 1;
                break;
            }
            
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                headers.insert(key, value);
            }
        }

        // Extract body
        let body = if body_start < lines.len() {
            lines[body_start..].join("\r\n").into_bytes()
        } else {
            Vec::new()
        };

        Ok(HttpResponse {
            version: "HTTP/1.1".to_string(),
            status,
            headers,
            body,
        })
    }
}

/// HTTP server handler trait
pub trait HttpHandler: Send + Sync {
    fn handle(&self, request: &HttpRequest) -> HttpResponse;
}

/// Simple function-based handler
pub struct FunctionHandler<F>
where
    F: Fn(&HttpRequest) -> HttpResponse + Send + Sync,
{
    handler: F,
}

impl<F> FunctionHandler<F>
where
    F: Fn(&HttpRequest) -> HttpResponse + Send + Sync,
{
    pub fn new(handler: F) -> Self {
        FunctionHandler { handler }
    }
}

impl<F> HttpHandler for FunctionHandler<F>
where
    F: Fn(&HttpRequest) -> HttpResponse + Send + Sync,
{
    fn handle(&self, request: &HttpRequest) -> HttpResponse {
        (self.handler)(request)
    }
}

/// HTTP server with routing support
pub struct HttpServer {
    routes: HashMap<(HttpMethod, String), Arc<dyn HttpHandler>>,
    middleware: Vec<Arc<dyn HttpHandler>>,
}

impl HttpServer {
    pub fn new() -> Self {
        HttpServer {
            routes: HashMap::new(),
            middleware: Vec::new(),
        }
    }

    pub fn route<H>(&mut self, method: HttpMethod, path: String, handler: H)
    where
        H: HttpHandler + 'static,
    {
        self.routes.insert((method, path), Arc::new(handler));
    }

    pub fn get<F>(&mut self, path: String, handler: F)
    where
        F: Fn(&HttpRequest) -> HttpResponse + Send + Sync + 'static,
    {
        self.route(HttpMethod::GET, path, FunctionHandler::new(handler));
    }

    pub fn post<F>(&mut self, path: String, handler: F)
    where
        F: Fn(&HttpRequest) -> HttpResponse + Send + Sync + 'static,
    {
        self.route(HttpMethod::POST, path, FunctionHandler::new(handler));
    }

    pub fn put<F>(&mut self, path: String, handler: F)
    where
        F: Fn(&HttpRequest) -> HttpResponse + Send + Sync + 'static,
    {
        self.route(HttpMethod::PUT, path, FunctionHandler::new(handler));
    }

    pub fn delete<F>(&mut self, path: String, handler: F)
    where
        F: Fn(&HttpRequest) -> HttpResponse + Send + Sync + 'static,
    {
        self.route(HttpMethod::DELETE, path, FunctionHandler::new(handler));
    }

    pub fn patch<F>(&mut self, path: String, handler: F)
    where
        F: Fn(&HttpRequest) -> HttpResponse + Send + Sync + 'static,
    {
        self.route(HttpMethod::PATCH, path, FunctionHandler::new(handler));
    }

    pub fn use_middleware<H>(&mut self, middleware: H)
    where
        H: HttpHandler + 'static,
    {
        self.middleware.push(Arc::new(middleware));
    }

    pub fn listen(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr)?;
        println!("HTTP server listening on {}", addr);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let routes = self.routes.clone();
                    let middleware = self.middleware.clone();
                    
                    thread::spawn(move || {
                        if let Err(e) = handle_connection(stream, routes, middleware) {
                            eprintln!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }

        Ok(())
    }

    pub fn handle_request(&self, request: &HttpRequest) -> HttpResponse {
        // Apply middleware (simplified - in real implementation would chain properly)
        for middleware in &self.middleware {
            // In a real implementation, middleware would be able to modify request/response
            let _response = middleware.handle(request);
        }

        // Find matching route
        if let Some(handler) = self.routes.get(&(request.method.clone(), request.path.clone())) {
            handler.handle(request)
        } else {
            HttpResponse::new(HttpStatus::NotFound)
                .with_text_body("Not Found".to_string())
        }
    }
}

fn handle_connection(
    mut stream: TcpStream,
    routes: HashMap<(HttpMethod, String), Arc<dyn HttpHandler>>,
    middleware: Vec<Arc<dyn HttpHandler>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [0; 4096];
    let bytes_read = stream.read(&mut buffer)?;
    
    let request_str = String::from_utf8_lossy(&buffer[..bytes_read]);
    let request = parse_http_request(&request_str)?;
    
    // Create temporary server to handle request
    let server = HttpServer {
        routes,
        middleware,
    };
    
    let response = server.handle_request(&request);
    let response_bytes = response.to_bytes();
    
    stream.write_all(&response_bytes)?;
    stream.flush()?;
    
    Ok(())
}

fn parse_http_request(request_str: &str) -> Result<HttpRequest, Box<dyn std::error::Error>> {
    let lines: Vec<&str> = request_str.split("\r\n").collect();
    
    if lines.is_empty() {
        return Err("Empty request".into());
    }

    // Parse request line
    let request_parts: Vec<&str> = lines[0].split_whitespace().collect();
    if request_parts.len() < 3 {
        return Err("Invalid request line".into());
    }

    let method = HttpMethod::from_str(request_parts[0])
        .ok_or("Invalid HTTP method")?;
    let path = request_parts[1].to_string();
    let version = request_parts[2].to_string();

    // Parse headers
    let mut headers = HashMap::new();
    let mut body_start = 0;
    
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.is_empty() {
            body_start = i + 1;
            break;
        }
        
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim().to_string();
            let value = line[colon_pos + 1..].trim().to_string();
            headers.insert(key, value);
        }
    }

    // Extract body
    let body = if body_start < lines.len() {
        lines[body_start..].join("\r\n").into_bytes()
    } else {
        Vec::new()
    };

    Ok(HttpRequest {
        method,
        path,
        version,
        headers,
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method_from_str() {
        assert_eq!(HttpMethod::from_str("GET"), Some(HttpMethod::GET));
        assert_eq!(HttpMethod::from_str("post"), Some(HttpMethod::POST));
        assert_eq!(HttpMethod::from_str("INVALID"), None);
    }

    #[test]
    fn test_http_status_code() {
        assert_eq!(HttpStatus::Ok.code(), 200);
        assert_eq!(HttpStatus::NotFound.code(), 404);
        assert_eq!(HttpStatus::InternalServerError.code(), 500);
    }

    #[test]
    fn test_http_request_creation() {
        let request = HttpRequest::new(HttpMethod::GET, "/test".to_string())
            .with_header("Content-Type".to_string(), "application/json".to_string())
            .with_json_body(r#"{"test": true}"#.to_string());

        assert_eq!(request.method, HttpMethod::GET);
        assert_eq!(request.path, "/test");
        assert_eq!(request.get_header("Content-Type"), Some(&"application/json".to_string()));
        assert_eq!(request.body_as_string().unwrap(), r#"{"test": true}"#);
    }

    #[test]
    fn test_http_response_creation() {
        let response = HttpResponse::new(HttpStatus::Ok)
            .with_json_body(r#"{"success": true}"#.to_string());

        assert_eq!(response.status, HttpStatus::Ok);
        assert_eq!(response.body_as_string().unwrap(), r#"{"success": true}"#);
        assert_eq!(response.headers.get("Content-Type"), Some(&"application/json".to_string()));
    }

    #[test]
    fn test_parse_http_request() {
        let request_str = "GET /test HTTP/1.1\r\nHost: example.com\r\nContent-Type: application/json\r\n\r\n{\"test\": true}";
        let request = parse_http_request(request_str).unwrap();

        assert_eq!(request.method, HttpMethod::GET);
        assert_eq!(request.path, "/test");
        assert_eq!(request.version, "HTTP/1.1");
        assert_eq!(request.get_header("Host"), Some(&"example.com".to_string()));
        assert_eq!(request.body_as_string().unwrap(), r#"{"test": true}"#);
    }

    #[test]
    fn test_http_server_routing() {
        let mut server = HttpServer::new();
        
        server.get("/test".to_string(), |_req| {
            HttpResponse::new(HttpStatus::Ok)
                .with_text_body("Hello, World!".to_string())
        });

        let request = HttpRequest::new(HttpMethod::GET, "/test".to_string());
        let response = server.handle_request(&request);

        assert_eq!(response.status, HttpStatus::Ok);
        assert_eq!(response.body_as_string().unwrap(), "Hello, World!");
    }
}