//! Local documentation server

use crate::Result;
use std::path::PathBuf;
use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use colored::*;

/// Local documentation server
pub struct DocServer {
    doc_dir: PathBuf,
    pub port: u16,
}

impl DocServer {
    pub fn new(doc_dir: PathBuf, port: u16) -> Self {
        Self { doc_dir, port }
    }

    /// Start the documentation server
    pub fn start(&self) -> Result<()> {
        let address = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&address)?;
        
        println!("{} Documentation server running at http://{}", 
                "Server".green().bold(), address);
        println!("Press Ctrl+C to stop the server");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let doc_dir = self.doc_dir.clone();
                    thread::spawn(move || {
                        if let Err(e) = handle_request(stream, &doc_dir) {
                            eprintln!("Error handling request: {}", e);
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
}

fn handle_request(mut stream: TcpStream, doc_dir: &PathBuf) -> Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;

    let request = String::from_utf8_lossy(&buffer[..]);
    let request_line = request.lines().next().unwrap_or("");
    
    // Parse the request
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        send_error_response(&mut stream, 400, "Bad Request")?;
        return Ok(());
    }

    let method = parts[0];
    let path = parts[1];

    if method != "GET" {
        send_error_response(&mut stream, 405, "Method Not Allowed")?;
        return Ok(());
    }

    // Determine the file to serve
    let file_path = if path == "/" {
        doc_dir.join("index.html")
    } else {
        let clean_path = path.trim_start_matches('/');
        doc_dir.join(clean_path)
    };

    // Security check: ensure the path is within the doc directory
    if !file_path.starts_with(doc_dir) {
        send_error_response(&mut stream, 403, "Forbidden")?;
        return Ok(());
    }

    // Serve the file
    if file_path.exists() && file_path.is_file() {
        serve_file(&mut stream, &file_path)?;
    } else {
        send_error_response(&mut stream, 404, "Not Found")?;
    }

    Ok(())
}

fn serve_file(stream: &mut TcpStream, file_path: &PathBuf) -> Result<()> {
    let contents = fs::read(file_path)?;
    let content_type = get_content_type(file_path);
    
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
        content_type,
        contents.len()
    );

    stream.write_all(response.as_bytes())?;
    stream.write_all(&contents)?;
    stream.flush()?;

    Ok(())
}

fn send_error_response(stream: &mut TcpStream, status_code: u16, status_text: &str) -> Result<()> {
    let body = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>{} {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        h1 {{ color: #dc2626; }}
    </style>
</head>
<body>
    <h1>{} {}</h1>
    <p>The requested resource could not be found or accessed.</p>
</body>
</html>"#,
        status_code, status_text, status_code, status_text
    );

    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
        status_code,
        status_text,
        body.len(),
        body
    );

    stream.write_all(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}

fn get_content_type(file_path: &PathBuf) -> &'static str {
    match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        _ => "text/plain; charset=utf-8",
    }
}