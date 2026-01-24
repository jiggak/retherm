use std::{thread, time::{Duration, Instant}};

use anyhow::Result;

use nest_backplate::*;

fn main() -> Result<()> {
    let mut backplate = BackplateConnection::open("/dev/ttyO2")?;

    // This triggers a constant stream of messages
    backplate.send_command(BackplateCmd::StatusRequest)?;

    let mut switch_state = false;
    let mut last_switch = Instant::now();

    loop {
        match backplate.read_message() {
            Ok(message) => {
                println!("{:?}", message);
            }
            Err(error) => {
                println!("Read error {}", error);
            }
        }

        if Instant::now() - last_switch > Duration::from_secs(3) {
            switch_state = !switch_state;
            last_switch = Instant::now();
            backplate.send_command(BackplateCmd::SwitchWire(Wire::Y1, switch_state))?;
        }

        thread::sleep(Duration::from_millis(250));
    }
}
