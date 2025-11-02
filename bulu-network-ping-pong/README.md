# Bulu Network Ping-Pong

A comprehensive network testing project for the Bulu programming language, demonstrating TCP and UDP networking capabilities.

## 🚀 Getting Started

### Run the Full Demo
```bash
lang run --source src/main.bu
```

### Run Individual Examples

**TCP Echo Server:**
```bash
lang run --source src/simple_tcp.bu
```
Then connect with: `telnet localhost 9090`

**UDP Echo Server:**
```bash
lang run --source src/simple_udp.bu
```

**UDP Client (to test UDP server):**
```bash
lang run --source src/udp_client.bu
```

## 📋 Examples Included

### 1. `main.bu` - Complete Ping-Pong Demo
- **TCP Ping-Pong**: Server and client exchange PING/PONG messages
- **UDP Ping-Pong**: Bidirectional UDP communication test
- **Goroutines**: Demonstrates concurrent server/client execution

### 2. `simple_tcp.bu` - TCP Echo Server
- Accepts multiple client connections
- Echoes back any message received
- Type `quit` to disconnect
- Test with: `telnet localhost 9090`

### 3. `simple_udp.bu` - UDP Echo Server
- Listens for UDP packets on port 9091
- Echoes back received messages
- Send `quit` to stop server

### 4. `udp_client.bu` - UDP Client
- Sends test messages to UDP server
- Receives and displays responses
- Automatically sends quit command

## 🔧 Network Features Tested

### TCP Features
- ✅ Server binding and listening
- ✅ Client connections
- ✅ Bidirectional data transfer
- ✅ Connection management
- ✅ Multiple client handling (goroutines)

### UDP Features
- ✅ Socket binding
- ✅ Packet sending/receiving
- ✅ Address resolution
- ✅ Bidirectional communication

### Networking Utilities
- ✅ NetAddr (IPv4/IPv6 addressing)
- ✅ Localhost and any address binding
- ✅ Port management
- ✅ Error handling

## 🧪 Testing the Network Stack

1. **Start with the full demo:**
   ```bash
   lang run --source src/main.bu
   ```

2. **Test TCP manually:**
   ```bash
   # Terminal 1: Start TCP server
   lang run --source src/simple_tcp.bu
   
   # Terminal 2: Connect with telnet
   telnet localhost 9090
   ```

3. **Test UDP manually:**
   ```bash
   # Terminal 1: Start UDP server
   lang run --source src/simple_udp.bu
   
   # Terminal 2: Run UDP client
   lang run --source src/udp_client.bu
   ```

## 📁 Project Structure

- `src/main.bu` - Complete ping-pong demo with TCP and UDP
- `src/simple_tcp.bu` - TCP echo server example
- `src/simple_udp.bu` - UDP echo server example  
- `src/udp_client.bu` - UDP client for testing
- `lang.toml` - Project configuration

## 🎯 Expected Output

The examples will show:
- 🔗 TCP connection establishment
- 📡 UDP packet exchange
- 📨 Message sending/receiving
- 📤 Response handling
- ✅ Successful network operations
- ❌ Error handling when issues occur

This project thoroughly tests Bulu's networking capabilities and serves as a reference for network programming in Bulu!
