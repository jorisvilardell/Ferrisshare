use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Parser)]
#[command(name = "ferris-cli")]
#[command(about = "CLI to communicate with ferrisshare listener", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send a file to a ferrisshare listener
    Send(SendArgs),
    /// Simple ping (HELLO) for testing
    Hello {
        /// remote address (host:port)
        #[arg(short, long, default_value = "127.0.0.1:9000")]
        addr: String,
        /// filename to announce
        filename: String,
        /// filesize in bytes
        filesize: u64,
    },
}

#[derive(Args)]
struct SendArgs {
    /// remote address
    #[arg(short, long, default_value = "127.0.0.1:9000")]
    addr: String,

    /// file to send
    #[arg(short, long)]
    file: PathBuf,

    /// block size (default 1024)
    #[arg(short = 'b', long, default_value_t = 1024u32)]
    block_size: u32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Hello {
            addr,
            filename,
            filesize,
        } => {
            let mut stream = TcpStream::connect(addr).await?;
            let line = format!("HELLO {} {}\n", filename, filesize);
            stream.write_all(line.as_bytes()).await?;
            // read response
            let mut buf = vec![0u8; 1024];
            let n = stream.read(&mut buf).await?;
            if n > 0 {
                let resp = String::from_utf8_lossy(&buf[..n]);
                println!("Reply: {}", resp.trim());
            }
        }
        Commands::Send(args) => {
            send_file(args).await?;
        }
    }

    Ok(())
}

async fn send_file(args: SendArgs) -> anyhow::Result<()> {
    let file = tokio::fs::File::open(&args.file).await?;
    let metadata = file.metadata().await?;
    let filesize = metadata.len();
    let filename = args
        .file
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?
        .to_string();

    let mut stream = TcpStream::connect(&args.addr).await?;

    // send HELLO
    let hello = format!("HELLO {} {}\n", filename, filesize);
    stream.write_all(hello.as_bytes()).await?;

    // wait for OK response (simple)
    let mut resp_buf = vec![0u8; 1024];
    let n = stream.read(&mut resp_buf).await?;
    if n > 0 {
        println!("Server: {}", String::from_utf8_lossy(&resp_buf[..n]).trim());
    }

    // stream file and send YEET commands + binary blocks
    let mut reader = tokio::fs::File::open(&args.file).await?;
    let mut index: u64 = 0;
    let mut buf = vec![0u8; args.block_size as usize];

    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 {
            break;
        }

        // send YEET header
        // checksum placeholder 0 for now
        let yeet = format!("YEET {} {} {}\n", index, n, 0);
        stream.write_all(yeet.as_bytes()).await?;

        // send binary block (raw bytes then newline)
        stream.write_all(&buf[..n]).await?;
        stream.write_all(b"\n").await?;

        // read ack
        let n = stream.read(&mut resp_buf).await?;
        if n > 0 {
            println!("Server: {}", String::from_utf8_lossy(&resp_buf[..n]).trim());
        }

        index += 1;
    }

    // send MISSION-ACCOMPLISHED
    stream.write_all(b"MISSION-ACCOMPLISHED\n").await?;
    let n = stream.read(&mut resp_buf).await?;
    if n > 0 {
        println!("Server: {}", String::from_utf8_lossy(&resp_buf[..n]).trim());
    }

    // send BYE-RIS
    stream.write_all(b"BYE-RIS\n").await?;

    Ok(())
}
