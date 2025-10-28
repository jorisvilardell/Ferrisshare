use std::convert::TryFrom;
use std::io::{self, BufRead};

mod core;

fn main() {
    use crate::core::domain::network::entities::ProtocolMessage;

    println!("FerrisShare REPL: tape une commande protocole (ligne vide pour quitter).");
    println!(
        "Exemples: HELLO foo.txt 42 | OK | NOPE raison | YEET 0 4096 12345 | OK-HOUSTEN 0 | MISSION-ACCOMPLISHED | SUCCESS | ERROR oops | BYE-RIS"
    );

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("I/O error: {e}");
                break;
            }
        };
        let trimmed = line.trim().to_string();
        if trimmed.is_empty() {
            println!("Bye.");
            break;
        }

        match ProtocolMessage::try_from(trimmed.as_str()) {
            Ok(msg) => println!("Parsed => {:?}", msg),
            Err(err) => println!("Parse error => {:?}", err),
        }
    }
}
