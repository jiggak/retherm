use std::time::{Duration, Instant};

use anyhow::Result;

use log::info;
use nest_backplate::*;

fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .parse_default_env()
        .init();

    let backplate = BackplateConnection::open("/dev/ttyO2")?;

    // This triggers a constant stream of messages
    backplate.send_command(BackplateCmd::StatusRequest)?;

    // toggle_circuit(backplate)
    sleep_testing(backplate)
}

fn toggle_circuit(mut backplate: BackplateConnection) -> Result<()> {
    let mut switch_state = false;
    let mut last_switch = Instant::now();

    loop {
        match backplate.read_message() {
            Ok(message) => {
                info!("{:?}", message);
            }
            Err(error) => {
                info!("Read error {}", error);
            }
        }

        if Instant::now() - last_switch > Duration::from_secs(3) {
            switch_state = !switch_state;
            last_switch = Instant::now();
            backplate.send_command(BackplateCmd::SwitchWire(Wire::Y1, switch_state))?;
        }
    }
}

fn sleep_testing(mut backplate: BackplateConnection) -> Result<()> {
    let mut start_time = Instant::now();
    let mut sent_quiet = false;

    loop {
        match backplate.read_message() {
            Ok(message) => {
                info!("{:?}", message);
            }
            Err(error) => {
                info!("Read error {}", error);
            }
        }

        if Instant::now() - start_time > Duration::from_secs(5) && !sent_quiet {
            sent_quiet = true;
            backplate.send_command(BackplateCmd::Quiet(10))?;
            info!("Be quiet for 10 sec");
        }

        if Instant::now() - start_time > Duration::from_secs(20) {
            start_time = Instant::now();
            backplate.send_command(BackplateCmd::StatusRequest)?;
            info!("Wakeup at 20s");
        }
    }
}
