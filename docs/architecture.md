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

## **Choice of Dependencies**

### Tokyo

The project uses **Tokio**, an asynchronous runtime for Rust, to manage networking operations and concurrency. Tokio provides powerful primitives such as `TcpStream`, `TcpListener`, and asynchronous task spawning (`tokio::spawn`), allowing efficient, non-blocking I/O.

This choice is motivated by several reasons:

1. **Asynchronous I/O efficiency** – Tokio leverages Rust’s `async/await` syntax to handle thousands of simultaneous client connections without blocking threads.
2. **Task scheduling and runtime** – Tokio includes a lightweight task scheduler that runs asynchronous functions concurrently on a single or multi-threaded runtime.
3. **Ecosystem integration** – Many crates (like `warp`, `hyper`, `reqwest`, `tokio-tungstenite`) are built on top of Tokio, ensuring good compatibility and extensibility.
4. **Fine-grained control** – Tokio allows precise management of I/O events, making it suitable for custom protocol implementations, chunked file transfer, and streaming optimizations.
5. **Performance and safety** – The runtime is highly optimized for low-latency operations, while maintaining Rust’s guarantees of memory safety and thread safety.

Without Tokio, the implementation would require manually managing threads and blocking I/O, which would be less efficient, harder to scale, and more error-prone.

## **FerrisShare File Transfer Protocol**

### **Overview**

The protocol defines a simple, **text-based command layer** over TCP for transferring a single file between two peers on the same network. It relies on TCP for reliable, ordered delivery, while adding **application-level commands** to coordinate the transfer, manage file chunks, and confirm completion. The connection is **bi-directional**, allowing the receiver to respond directly through the same TCP stream.

### **Protocol Commands**

| Command                  | Sender | Arguments                                              | Response                   | Description                                                                                      |
| ------------------------ | ------ | ------------------------------------------------------ | -------------------------- | ------------------------------------------------------------------------------------------------ |
| **HELLO**                | Client | `<filename> <filesize>`                                | `OK` / `NOPE <reason>`     | Initiates the file transfer and informs the receiver about the file name and size.               |
| **OK**                   | Server | —                                                      | —                          | Confirms acceptance of the file transfer.                                                        |
| **NOPE**                 | Server | `<reason>`                                             | —                          | Refuses the transfer (e.g., file exists, insufficient space).                                    |
| **YEET**                 | Client | `<block_index> <block_size> <check_sum>` + binary data | `OK-HOUSTEN <block_index>` | Sends one block of the file to the receiver. Blocks are fixed or variable size.                  |
| **OK-HOUSTEN**           | Server | `<block_index>`                                        | —                          | Confirms the block was received and written correctly. Optional but recommended for integrity.   |
| **MISSION-ACCOMPLISHED** | Client | —                                                      | `SUCCESS` / `ERROR`        | Marks the end of file transmission. The server verifies that all blocks were received correctly. |
| **BYE-RIS**              | Either | —                                                      | —                          | Gracefully terminates or cancels the transfer.                                                   |

### **Notes**

- **TCP guarantees delivery**, but `CONFIRM` adds **application-level integrity verification**.
- **File is transferred in blocks (blobs)** to allow streaming of large files without memory overload.
- **Bi-directional communication** is handled over the same TCP connection; no additional socket is needed.
- Protocol is designed to be **minimal, readable, and extensible** for future features (resume, hash verification, multi-file support).
