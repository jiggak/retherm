use std::{thread, time::Duration};

use anyhow::Result;

use nest_backplate::*;

fn main() -> Result<()> {
    let mut backplate = BackplateConnection::open("/dev/ttyO2")?;

    // This triggers a constant stream of messages
    backplate.send_command(BackplateCmd::StatusRequest)?;

    loop {
        match backplate.read_message() {
            Ok(message) => {
                if let Some(message) = message {
                    println!("{:?}", message);
                }
            }
            Err(error) => {
                println!("Read error {}", error);
            }
        }

        thread::sleep(Duration::from_millis(250));
    }
}
