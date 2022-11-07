use std::fmt::Error;
use shadow_rs::{shadow, Format};
use color_eyre::eyre::{self, WrapErr};
use eyre::*;
use tracing::instrument;

shadow!(build); //Required for shadow-rs to work

#[instrument]
fn main() -> eyre::Result<()> {
    color_eyre::install()?; //Set up eyre (with colours) for error handling

    println!("Hello, world!");
    let string = "string";
    
    return Err(eyre!("Test lol: {} {asd} {string}", 1, asd=69420));
}