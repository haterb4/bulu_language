// Comprehensive demo of networking and data format modules
// This example demonstrates std.http, std.net, std.json, std.xml, and std.csv

use bulu::std::{http, net, json, xml, csv};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Bulu Standard Library: Networking and Data Formats Demo ===\n");

    // Demo JSON functionality
    demo_json();
    
    // Demo XML functionality
    demo_xml();
    
    // Demo CSV functionality
    demo_csv();
    
    // Demo networking functionality
    demo_networking();
    
    // Demo HTTP functionality
    demo_http();
    
    println!("=== Demo completed successfully! ===");
}

fn demo_json() {
    println!("🔧 JSON Module Demo");
    println!("-------------------");
    
    // Create JSON data
    let mut user_data = json::Json::object();
    user_data.insert("name".to_string(), json::Json::from_str("Alice"));
    user_data.insert("age".to_string(), json::Json::from_i64(30));
    user_data.insert("active".to_string(), json::Json::from_bool(true));
    
    let mut skills = json::Json::array();
    skills.push(json::Json::from_str("Rust")).unwrap();
    skills.push(json::Json::from_str("JavaScript")).unwrap();
    skills.push(json::Json::from_str("Python")).unwrap();
    user_data.insert("skills".to_string(), skills);
    
    // Serialize to JSON
    let json_string = json::Json::stringify(&user_data);
    println!("Compact JSON: {}", json_string);
    
    let pretty_json = json::Json::stringify_pretty(&user_data);
    println!("Pretty JSON:\n{}", pretty_json);
    
    // Parse JSON back
    let parsed = json::Json::parse(&json_string).unwrap();
    println!("Parsed name: {}", parsed.get("name").unwrap().as_str().unwrap());
    println!("Parsed age: {}", parsed.get("age").unwrap().as_i64().unwrap());
    
    println!();
}

fn demo_xml() {
    println!("📄 XML Module Demo");
    println!("------------------");
    
    // Create XML document
    let mut document = xml::XmlDocument::new();
    
    // Add XML declaration
    document.set_declaration(xml::XmlNode::declaration(
        "1.0".to_string(),
        Some("UTF-8".to_string()),
        None
    ));
    
    // Create root element
    let mut users = xml::XmlNode::element("users".to_string());
    
    // Create user elements
    let mut user1 = xml::XmlNode::element("user".to_string());
    user1.set_attribute("id".to_string(), "1".to_string()).unwrap();
    
    let mut name1 = xml::XmlNode::element("name".to_string());
    name1.add_child(xml::XmlNode::text("Alice".to_string())).unwrap();
    user1.add_child(name1).unwrap();
    
    let mut email1 = xml::XmlNode::element("email".to_string());
    email1.add_child(xml::XmlNode::text("alice@example.com".to_string())).unwrap();
    user1.add_child(email1).unwrap();
    
    users.add_child(user1).unwrap();
    
    let mut user2 = xml::XmlNode::element("user".to_string());
    user2.set_attribute("id".to_string(), "2".to_string()).unwrap();
    
    let mut name2 = xml::XmlNode::element("name".to_string());
    name2.add_child(xml::XmlNode::text("Bob".to_string())).unwrap();
    user2.add_child(name2).unwrap();
    
    let mut email2 = xml::XmlNode::element("email".to_string());
    email2.add_child(xml::XmlNode::text("bob@example.com".to_string())).unwrap();
    user2.add_child(email2).unwrap();
    
    users.add_child(user2).unwrap();
    document.set_root(users);
    
    // Serialize XML
    let xml_string = xml::Xml::stringify_pretty(&document);
    println!("Generated XML:\n{}", xml_string);
    
    // Parse XML back
    let parsed_doc = xml::Xml::parse(&xml_string).unwrap();
    let root = parsed_doc.root.unwrap();
    let user_nodes = root.find_children("user");
    
    println!("Parsed {} users:", user_nodes.len());
    for user in user_nodes {
        let id = user.get_attribute("id").unwrap();
        let name = user.find_child("name").unwrap().inner_text();
        let email = user.find_child("email").unwrap().inner_text();
        println!("  User {}: {} ({})", id, name, email);
    }
    
    println!();
}

fn demo_csv() {
    println!("📊 CSV Module Demo");
    println!("------------------");
    
    // Create CSV document with headers
    let mut document = csv::CsvDocument::with_headers(vec![
        "Name".to_string(),
        "Age".to_string(),
        "Department".to_string(),
        "Salary".to_string(),
    ]);
    
    // Add records
    let employees = vec![
        vec!["Alice Johnson".to_string(), "30".to_string(), "Engineering".to_string(), "75000".to_string()],
        vec!["Bob Smith".to_string(), "25".to_string(), "Marketing".to_string(), "55000".to_string()],
        vec!["Carol Davis".to_string(), "35".to_string(), "Sales".to_string(), "65000".to_string()],
    ];
    
    for emp_data in employees {
        let record = csv::CsvRecord::from_fields(emp_data);
        document.add_record(record);
    }
    
    // Write CSV
    let csv_string = csv::Csv::write(&document);
    println!("Generated CSV:\n{}", csv_string);
    
    // Parse CSV back
    let parsed_doc = csv::Csv::parse_with_headers(&csv_string).unwrap();
    println!("Parsed {} employees:", parsed_doc.len());
    
    for i in 0..parsed_doc.len() {
        let name = parsed_doc.get_field_by_name(i, "Name").unwrap().unwrap();
        let age = parsed_doc.get_field_by_name(i, "Age").unwrap().unwrap();
        let dept = parsed_doc.get_field_by_name(i, "Department").unwrap().unwrap();
        let salary = parsed_doc.get_field_by_name(i, "Salary").unwrap().unwrap();
        println!("  {}: {} years old, {} department, ${}", name, age, dept, salary);
    }
    
    // Convert to maps for easier processing
    let maps = parsed_doc.to_maps().unwrap();
    let total_salary: i32 = maps.iter()
        .map(|emp| emp.get("Salary").unwrap().parse::<i32>().unwrap())
        .sum();
    println!("Total salary budget: ${}", total_salary);
    
    println!();
}

fn demo_networking() {
    println!("🌐 Networking Module Demo");
    println!("-------------------------");
    
    // Test network address creation
    let localhost = net::NetAddr::localhost_ipv4(8080);
    println!("Created localhost address: port {}", localhost.port());
    
    let domain_addr = net::NetAddr::new_domain("example.com".to_string(), 80);
    println!("Created domain address: port {}", domain_addr.port());
    
    // Test network utilities
    println!("Checking port availability...");
    let available_port = net::NetUtils::find_available_port(8000).unwrap();
    println!("Found available port: {}", available_port);
    
    // Test IP parsing
    let ip = net::NetUtils::parse_ip("127.0.0.1").unwrap();
    println!("Parsed IP: {} (is loopback: {})", ip, ip.is_loopback());
    
    // Demo TCP server and client
    println!("Starting TCP echo server...");
    let server_addr = net::NetAddr::localhost_ipv4(0); // Use port 0 for auto-assignment
    let server = net::TcpServer::bind(server_addr).unwrap();
    let server_port = server.local_addr().port();
    println!("Server listening on port {}", server_port);
    
    // Start server in background thread
    let server_handle = thread::spawn(move || {
        if let Ok(mut connection) = server.accept() {
            let mut buffer = [0; 1024];
            if let Ok(n) = connection.read(&mut buffer) {
                let _ = connection.write_all(&buffer[..n]);
            }
        }
    });
    
    // Give server time to start
    thread::sleep(Duration::from_millis(10));
    
    // Connect as client
    let client_addr = net::NetAddr::localhost_ipv4(server_port);
    let mut client = net::TcpConnection::connect(client_addr).unwrap();
    
    let message = b"Hello, TCP!";
    client.write_all(message).unwrap();
    
    let mut response = [0; 1024];
    let n = client.read(&mut response).unwrap();
    let echo = String::from_utf8_lossy(&response[..n]);
    println!("TCP Echo response: {}", echo);
    
    // Wait for server thread to complete
    let _ = server_handle.join();
    
    // Demo UDP communication
    println!("Testing UDP communication...");
    let udp1 = net::UdpConnection::bind(net::NetAddr::localhost_ipv4(0)).unwrap();
    let udp2 = net::UdpConnection::bind(net::NetAddr::localhost_ipv4(0)).unwrap();
    
    let port1 = udp1.local_addr().port();
    let port2 = udp2.local_addr().port();
    
    let udp_message = b"Hello, UDP!";
    udp1.send_to(udp_message, net::NetAddr::localhost_ipv4(port2)).unwrap();
    
    let mut udp_buffer = [0; 1024];
    let (n, sender_addr) = udp2.recv_from(&mut udp_buffer).unwrap();
    let udp_response = String::from_utf8_lossy(&udp_buffer[..n]);
    println!("UDP message received: {} from port {}", udp_response, sender_addr.port());
    
    println!();
}

fn demo_http() {
    println!("🌍 HTTP Module Demo");
    println!("-------------------");
    
    // Create HTTP client
    let client = http::HttpClient::new()
        .with_default_header("User-Agent".to_string(), "Bulu-Demo/1.0".to_string());
    
    // Create HTTP server
    let mut server = http::HttpServer::new();
    
    // Add routes
    server.get("/".to_string(), |_req| {
        http::HttpResponse::new(http::HttpStatus::Ok)
            .with_text_body("Welcome to Bulu HTTP Server!".to_string())
    });
    
    server.get("/api/users".to_string(), |_req| {
        let users = json::Json::parse(r#"[
            {"id": 1, "name": "Alice", "email": "alice@example.com"},
            {"id": 2, "name": "Bob", "email": "bob@example.com"}
        ]"#).unwrap();
        
        http::HttpResponse::new(http::HttpStatus::Ok)
            .with_json_body(json::Json::stringify(&users))
    });
    
    server.post("/api/users".to_string(), |req| {
        let body = req.body_as_string().unwrap_or_default();
        println!("Received POST data: {}", body);
        
        let response_data = json::Json::parse(r#"{"id": 3, "status": "created"}"#).unwrap();
        http::HttpResponse::new(http::HttpStatus::Created)
            .with_json_body(json::Json::stringify(&response_data))
    });
    
    // Test HTTP requests and responses
    println!("Testing HTTP request/response handling...");
    
    // Test GET request
    let get_request = http::HttpRequest::new(http::HttpMethod::GET, "/".to_string());
    let get_response = server.handle_request(&get_request);
    println!("GET / -> Status: {}, Body: {}", 
        get_response.status.code(), 
        get_response.body_as_string().unwrap()
    );
    
    // Test API GET request
    let api_request = http::HttpRequest::new(http::HttpMethod::GET, "/api/users".to_string());
    let api_response = server.handle_request(&api_request);
    println!("GET /api/users -> Status: {}", api_response.status.code());
    
    let users_json = api_response.body_as_string().unwrap();
    let users = json::Json::parse(&users_json).unwrap();
    println!("Users data: {}", json::Json::stringify_pretty(&users));
    
    // Test POST request
    let post_data = json::Json::parse(r#"{"name": "Charlie", "email": "charlie@example.com"}"#).unwrap();
    let post_request = http::HttpRequest::new(http::HttpMethod::POST, "/api/users".to_string())
        .with_json_body(json::Json::stringify(&post_data));
    
    let post_response = server.handle_request(&post_request);
    println!("POST /api/users -> Status: {}, Body: {}", 
        post_response.status.code(),
        post_response.body_as_string().unwrap()
    );
    
    // Test 404 response
    let not_found_request = http::HttpRequest::new(http::HttpMethod::GET, "/nonexistent".to_string());
    let not_found_response = server.handle_request(&not_found_request);
    println!("GET /nonexistent -> Status: {}", not_found_response.status.code());
    
    println!();
}