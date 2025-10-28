# FerrisShare - P2P File Transfer Protocol

## 1. Project Definition

### What is FerrisShare?

FerrisShare is a peer-to-peer (P2P) file transfer command-line tool that enables direct file sharing between two computers on the same network without requiring a central server. Built in Rust, it prioritizes reliability, concurrent connection handling, and a simple custom protocol for efficient file transfers.

### Goals

The primary goals of this project are:

1. **Direct P2P Transfer**: Enable users to send files directly to another machine using only an IP address and port number
2. **Bidirectional Operation**: Support both sending and receiving modes within a single binary
3. **Concurrent Handling**: Allow a receiver to accept multiple simultaneous file transfers from different senders
4. **Reliability**: Implement a simple protocol with handshake verification to ensure successful transfers
5. **Simplicity**: Provide a straightforward CLI interface similar to common networking tools

### Non-Goals

- **Discovery Protocol**: The sender must know the receiver's IP address (no automatic peer discovery)
- **Encryption**: File transfers are not encrypted (local network trust assumed)
- **Resume Support**: Interrupted transfers cannot be resumed
- **Multi-file Transfers**: Each transfer handles exactly one file
