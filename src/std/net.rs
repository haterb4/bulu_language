// TCP/UDP networking functionality for the Bulu programming language
// Requirements: 7.2.2

use std::io::{Read, Write, Result as IoResult};
use std::net::{
    TcpListener, TcpStream, UdpSocket, SocketAddr, ToSocketAddrs,
    Ipv4Addr, Ipv6Addr, IpAddr,
};
use std::time::Duration;

/// Network address types
#[derive(Debug, Clone, PartialEq)]
pub enum NetAddr {
    Ipv4(Ipv4Addr, u16),
    Ipv6(Ipv6Addr, u16),
    Domain(String, u16),
}

impl NetAddr {
    pub fn new_ipv4(ip: [u8; 4], port: u16) -> Self {
        NetAddr::Ipv4(Ipv4Addr::from(ip), port)
    }

    pub fn new_ipv6(ip: [u16; 8], port: u16) -> Self {
        NetAddr::Ipv6(Ipv6Addr::from(ip), port)
    }

    pub fn new_domain(domain: String, port: u16) -> Self {
        NetAddr::Domain(domain, port)
    }

    pub fn localhost_ipv4(port: u16) -> Self {
        NetAddr::Ipv4(Ipv4Addr::LOCALHOST, port)
    }

    pub fn localhost_ipv6(port: u16) -> Self {
        NetAddr::Ipv6(Ipv6Addr::LOCALHOST, port)
    }

    pub fn any_ipv4(port: u16) -> Self {
        NetAddr::Ipv4(Ipv4Addr::UNSPECIFIED, port)
    }

    pub fn any_ipv6(port: u16) -> Self {
        NetAddr::Ipv6(Ipv6Addr::UNSPECIFIED, port)
    }

    pub fn to_socket_addr(&self) -> Result<SocketAddr, Box<dyn std::error::Error>> {
        match self {
            NetAddr::Ipv4(ip, port) => Ok(SocketAddr::new(IpAddr::V4(*ip), *port)),
            NetAddr::Ipv6(ip, port) => Ok(SocketAddr::new(IpAddr::V6(*ip), *port)),
            NetAddr::Domain(domain, port) => {
                let addr_str = format!("{}:{}", domain, port);
                let mut addrs = addr_str.to_socket_addrs()?;
                addrs.next().ok_or("Failed to resolve domain".into())
            }
        }
    }

    pub fn port(&self) -> u16 {
        match self {
            NetAddr::Ipv4(_, port) => *port,
            NetAddr::Ipv6(_, port) => *port,
            NetAddr::Domain(_, port) => *port,
        }
    }
}

/// TCP connection wrapper
pub struct TcpConnection {
    stream: TcpStream,
    local_addr: SocketAddr,
    peer_addr: SocketAddr,
}

impl TcpConnection {
    pub fn connect(addr: NetAddr) -> Result<Self, Box<dyn std::error::Error>> {
        let socket_addr = addr.to_socket_addr()?;
        let stream = TcpStream::connect(socket_addr)?;
        let local_addr = stream.local_addr()?;
        let peer_addr = stream.peer_addr()?;

        Ok(TcpConnection {
            stream,
            local_addr,
            peer_addr,
        })
    }

    pub fn connect_timeout(addr: NetAddr, timeout: Duration) -> Result<Self, Box<dyn std::error::Error>> {
        let socket_addr = addr.to_socket_addr()?;
        let stream = TcpStream::connect_timeout(&socket_addr, timeout)?;
        let local_addr = stream.local_addr()?;
        let peer_addr = stream.peer_addr()?;

        Ok(TcpConnection {
            stream,
            local_addr,
            peer_addr,
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }

    pub fn set_read_timeout(&mut self, timeout: Option<Duration>) -> IoResult<()> {
        self.stream.set_read_timeout(timeout)
    }

    pub fn set_write_timeout(&mut self, timeout: Option<Duration>) -> IoResult<()> {
        self.stream.set_write_timeout(timeout)
    }

    pub fn set_nodelay(&mut self, nodelay: bool) -> IoResult<()> {
        self.stream.set_nodelay(nodelay)
    }

    pub fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.stream.read(buf)
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> IoResult<()> {
        self.stream.read_exact(buf)
    }

    pub fn read_to_end(&mut self, buf: &mut Vec<u8>) -> IoResult<usize> {
        self.stream.read_to_end(buf)
    }

    pub fn read_to_string(&mut self, buf: &mut String) -> IoResult<usize> {
        self.stream.read_to_string(buf)
    }

    pub fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.stream.write(buf)
    }

    pub fn write_all(&mut self, buf: &[u8]) -> IoResult<()> {
        self.stream.write_all(buf)
    }

    pub fn flush(&mut self) -> IoResult<()> {
        self.stream.flush()
    }

    pub fn shutdown(&mut self, how: std::net::Shutdown) -> IoResult<()> {
        self.stream.shutdown(how)
    }

    pub fn try_clone(&self) -> IoResult<TcpConnection> {
        let stream = self.stream.try_clone()?;
        Ok(TcpConnection {
            stream,
            local_addr: self.local_addr,
            peer_addr: self.peer_addr,
        })
    }
}

/// TCP server wrapper
pub struct TcpServer {
    listener: TcpListener,
    local_addr: SocketAddr,
}

impl TcpServer {
    pub fn bind(addr: NetAddr) -> Result<Self, Box<dyn std::error::Error>> {
        let socket_addr = addr.to_socket_addr()?;
        let listener = TcpListener::bind(socket_addr)?;
        let local_addr = listener.local_addr()?;

        Ok(TcpServer {
            listener,
            local_addr,
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    pub fn accept(&self) -> IoResult<TcpConnection> {
        let (stream, peer_addr) = self.listener.accept()?;
        let local_addr = stream.local_addr()?;

        Ok(TcpConnection {
            stream,
            local_addr,
            peer_addr,
        })
    }

    pub fn incoming(&self) -> TcpIncoming {
        TcpIncoming {
            listener: &self.listener,
        }
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> IoResult<()> {
        self.listener.set_nonblocking(nonblocking)
    }

    pub fn try_clone(&self) -> IoResult<TcpServer> {
        let listener = self.listener.try_clone()?;
        Ok(TcpServer {
            listener,
            local_addr: self.local_addr,
        })
    }
}

/// Iterator for incoming TCP connections
pub struct TcpIncoming<'a> {
    listener: &'a TcpListener,
}

impl<'a> Iterator for TcpIncoming<'a> {
    type Item = IoResult<TcpConnection>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.listener.accept() {
            Ok((stream, peer_addr)) => {
                match stream.local_addr() {
                    Ok(local_addr) => Some(Ok(TcpConnection {
                        stream,
                        local_addr,
                        peer_addr,
                    })),
                    Err(e) => Some(Err(e)),
                }
            }
            Err(e) => Some(Err(e)),
        }
    }
}

/// UDP socket wrapper
pub struct UdpConnection {
    socket: UdpSocket,
    local_addr: SocketAddr,
}

impl UdpConnection {
    pub fn bind(addr: NetAddr) -> Result<Self, Box<dyn std::error::Error>> {
        let socket_addr = addr.to_socket_addr()?;
        let socket = UdpSocket::bind(socket_addr)?;
        let local_addr = socket.local_addr()?;

        Ok(UdpConnection {
            socket,
            local_addr,
        })
    }

    pub fn connect(&self, addr: NetAddr) -> Result<(), Box<dyn std::error::Error>> {
        let socket_addr = addr.to_socket_addr()?;
        self.socket.connect(socket_addr)?;
        Ok(())
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    pub fn peer_addr(&self) -> IoResult<SocketAddr> {
        self.socket.peer_addr()
    }

    pub fn send(&self, buf: &[u8]) -> IoResult<usize> {
        self.socket.send(buf)
    }

    pub fn send_to(&self, buf: &[u8], addr: NetAddr) -> Result<usize, Box<dyn std::error::Error>> {
        let socket_addr = addr.to_socket_addr()?;
        Ok(self.socket.send_to(buf, socket_addr)?)
    }

    pub fn recv(&self, buf: &mut [u8]) -> IoResult<usize> {
        self.socket.recv(buf)
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> IoResult<(usize, SocketAddr)> {
        self.socket.recv_from(buf)
    }

    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> IoResult<()> {
        self.socket.set_read_timeout(timeout)
    }

    pub fn set_write_timeout(&self, timeout: Option<Duration>) -> IoResult<()> {
        self.socket.set_write_timeout(timeout)
    }

    pub fn set_broadcast(&self, broadcast: bool) -> IoResult<()> {
        self.socket.set_broadcast(broadcast)
    }

    pub fn set_multicast_loop_v4(&self, multicast_loop_v4: bool) -> IoResult<()> {
        self.socket.set_multicast_loop_v4(multicast_loop_v4)
    }

    pub fn set_multicast_ttl_v4(&self, multicast_ttl_v4: u32) -> IoResult<()> {
        self.socket.set_multicast_ttl_v4(multicast_ttl_v4)
    }

    pub fn set_ttl(&self, ttl: u32) -> IoResult<()> {
        self.socket.set_ttl(ttl)
    }

    pub fn join_multicast_v4(&self, multiaddr: &Ipv4Addr, interface: &Ipv4Addr) -> IoResult<()> {
        self.socket.join_multicast_v4(multiaddr, interface)
    }

    pub fn leave_multicast_v4(&self, multiaddr: &Ipv4Addr, interface: &Ipv4Addr) -> IoResult<()> {
        self.socket.leave_multicast_v4(multiaddr, interface)
    }

    pub fn join_multicast_v6(&self, multiaddr: &Ipv6Addr, interface: u32) -> IoResult<()> {
        self.socket.join_multicast_v6(multiaddr, interface)
    }

    pub fn leave_multicast_v6(&self, multiaddr: &Ipv6Addr, interface: u32) -> IoResult<()> {
        self.socket.leave_multicast_v6(multiaddr, interface)
    }

    pub fn try_clone(&self) -> IoResult<UdpConnection> {
        let socket = self.socket.try_clone()?;
        Ok(UdpConnection {
            socket,
            local_addr: self.local_addr,
        })
    }
}

/// Network utilities
pub struct NetUtils;

impl NetUtils {
    /// Resolve a hostname to IP addresses
    pub fn resolve_host(hostname: &str) -> Result<Vec<IpAddr>, Box<dyn std::error::Error>> {
        let addrs: Vec<SocketAddr> = format!("{}:0", hostname).to_socket_addrs()?.collect();
        Ok(addrs.into_iter().map(|addr| addr.ip()).collect())
    }

    /// Check if a port is available on localhost
    pub fn is_port_available(port: u16) -> bool {
        TcpListener::bind(("127.0.0.1", port)).is_ok()
    }

    /// Find an available port starting from the given port
    pub fn find_available_port(start_port: u16) -> Option<u16> {
        for port in start_port..=65535 {
            if Self::is_port_available(port) {
                return Some(port);
            }
        }
        None
    }

    /// Parse an IP address string
    pub fn parse_ip(ip_str: &str) -> Result<IpAddr, std::net::AddrParseError> {
        ip_str.parse()
    }

    /// Parse a socket address string
    pub fn parse_socket_addr(addr_str: &str) -> Result<SocketAddr, std::net::AddrParseError> {
        addr_str.parse()
    }

    /// Get the local IP address (simplified - returns first non-loopback address)
    pub fn get_local_ip() -> Result<IpAddr, Box<dyn std::error::Error>> {
        // This is a simplified implementation
        // In a real implementation, we would enumerate network interfaces
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect("8.8.8.8:80")?;
        Ok(socket.local_addr()?.ip())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_net_addr_creation() {
        let addr = NetAddr::new_ipv4([127, 0, 0, 1], 8080);
        assert_eq!(addr.port(), 8080);

        let addr = NetAddr::localhost_ipv4(3000);
        assert_eq!(addr.port(), 3000);

        let addr = NetAddr::new_domain("example.com".to_string(), 80);
        assert_eq!(addr.port(), 80);
    }

    #[test]
    fn test_net_addr_to_socket_addr() {
        let addr = NetAddr::localhost_ipv4(8080);
        let socket_addr = addr.to_socket_addr().unwrap();
        assert_eq!(socket_addr.port(), 8080);
        assert!(socket_addr.ip().is_loopback());
    }

    #[test]
    fn test_tcp_server_bind() {
        let addr = NetAddr::localhost_ipv4(0); // Use port 0 for automatic assignment
        let server = TcpServer::bind(addr).unwrap();
        assert!(server.local_addr().port() > 0);
    }

    #[test]
    fn test_tcp_connection() {
        // Start a simple echo server
        let server_addr = NetAddr::localhost_ipv4(0);
        let server = TcpServer::bind(server_addr).unwrap();
        let server_port = server.local_addr().port();

        thread::spawn(move || {
            if let Ok(mut conn) = server.accept() {
                let mut buffer = [0; 1024];
                if let Ok(n) = conn.read(&mut buffer) {
                    let _ = conn.write_all(&buffer[..n]);
                }
            }
        });

        // Give server time to start
        thread::sleep(Duration::from_millis(10));

        // Connect to server
        let client_addr = NetAddr::localhost_ipv4(server_port);
        let mut client = TcpConnection::connect(client_addr).unwrap();

        // Send data
        let test_data = b"Hello, TCP!";
        client.write_all(test_data).unwrap();

        // Read response
        let mut buffer = [0; 1024];
        let n = client.read(&mut buffer).unwrap();
        assert_eq!(&buffer[..n], test_data);
    }

    #[test]
    fn test_udp_connection() {
        let addr = NetAddr::localhost_ipv4(0);
        let socket = UdpConnection::bind(addr).unwrap();
        assert!(socket.local_addr().port() > 0);
    }

    #[test]
    fn test_udp_send_recv() {
        // Create two UDP sockets
        let addr1 = NetAddr::localhost_ipv4(0);
        let socket1 = UdpConnection::bind(addr1).unwrap();
        let port1 = socket1.local_addr().port();

        let addr2 = NetAddr::localhost_ipv4(0);
        let socket2 = UdpConnection::bind(addr2).unwrap();
        let port2 = socket2.local_addr().port();

        // Send from socket1 to socket2
        let test_data = b"Hello, UDP!";
        let target_addr = NetAddr::localhost_ipv4(port2);
        socket1.send_to(test_data, target_addr).unwrap();

        // Receive on socket2
        let mut buffer = [0; 1024];
        let (n, sender_addr) = socket2.recv_from(&mut buffer).unwrap();
        assert_eq!(&buffer[..n], test_data);
        assert_eq!(sender_addr.port(), port1);
    }

    #[test]
    fn test_net_utils_port_availability() {
        // Port 0 should always be available for binding (auto-assigned)
        assert!(NetUtils::is_port_available(0));
        
        // Find an available port
        let port = NetUtils::find_available_port(8000).unwrap();
        assert!(port >= 8000);
    }

    #[test]
    fn test_net_utils_ip_parsing() {
        let ip = NetUtils::parse_ip("127.0.0.1").unwrap();
        assert!(ip.is_loopback());

        let ip = NetUtils::parse_ip("::1").unwrap();
        assert!(ip.is_loopback());

        let socket_addr = NetUtils::parse_socket_addr("127.0.0.1:8080").unwrap();
        assert_eq!(socket_addr.port(), 8080);
    }

    #[test]
    fn test_tcp_connection_timeout() {
        // Try to connect to a non-existent server with timeout
        let addr = NetAddr::new_ipv4([192, 0, 2, 1], 12345); // RFC 5737 test address
        let timeout = Duration::from_millis(100);
        
        let start = std::time::Instant::now();
        let result = TcpConnection::connect_timeout(addr, timeout);
        let elapsed = start.elapsed();
        
        assert!(result.is_err());
        assert!(elapsed >= timeout);
        assert!(elapsed < timeout + Duration::from_millis(100)); // Allow some margin
    }

    #[test]
    fn test_udp_multicast() {
        let addr = NetAddr::any_ipv4(0);
        let socket = UdpConnection::bind(addr).unwrap();
        
        let multicast_addr = Ipv4Addr::new(224, 0, 0, 1);
        let interface_addr = Ipv4Addr::new(127, 0, 0, 1);
        
        // These operations should not fail on most systems
        let _ = socket.join_multicast_v4(&multicast_addr, &interface_addr);
        let _ = socket.leave_multicast_v4(&multicast_addr, &interface_addr);
    }
}