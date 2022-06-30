#![cfg(feature = "tokio")]

use std::error;

use super::helpers::{get_conn_addr, init_logger, PASSWORD};

#[tokio::test]
async fn simple() -> Result<(), Box<dyn error::Error>> {
    init_logger();

    Ok(())
}
