use anyhow::Result;

use nest_backplate::*;

fn main() -> Result<()> {
    open("/dev/ttyO2")?;

    Ok(())
}
