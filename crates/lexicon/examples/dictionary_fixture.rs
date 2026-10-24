//! Writes and verifies the complete S02 composed dictionary fixture.
#[path = "support/dictionary_fixture.rs"]
mod fixture_support;
use std::path::PathBuf;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: dictionary_fixture OUTPUT.json")?;
    fixture_support::write_and_reload(&path)?;
    println!(
        "verified 9 captures, 20 occurrences, 14 entries, 20 assertions and all twelve metadata areas at {}",
        path.display()
    );
    Ok(())
}
