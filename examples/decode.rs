use anyhow::Result;
use qoi::decode::QOI;

fn main() -> Result<()> {
    let dogfood = include_bytes!("../images/baboon.qoi");

    let (_, qoi) = QOI::parse(dogfood)?;

    let _pixels = qoi.into_pixels();

    Ok(())
}