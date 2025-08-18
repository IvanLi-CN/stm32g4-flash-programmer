use anyhow::{Context, Result};
use flash_protocol::*;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;
use tokio_serial::SerialStream;

pub struct SerialConnection {
    port: SerialStream,
}

impl SerialConnection {
    pub async fn new(port_name: &str, baud_rate: u32) -> Result<Self> {
        let port = SerialStream::open(&tokio_serial::new(port_name, baud_rate))
            .with_context(|| format!("Failed to open serial port: {}", port_name))?;

        Ok(Self { port })
    }

    pub async fn send_packet(&mut self, packet: &Packet) -> Result<()> {
        let data = packet.to_bytes();

        // Send packet
        self.port
            .write_all(&data)
            .await
            .context("Failed to write packet to serial port")?;

        Ok(())
    }

    pub async fn receive_response(&mut self) -> Result<Response> {
        let mut buffer = Vec::new();
        let mut temp_buf = [0u8; 1024];

        // Read response with timeout
        loop {
            match timeout(Duration::from_secs(30), self.port.read(&mut temp_buf)).await {
                Ok(Ok(n)) if n > 0 => {
                    buffer.extend_from_slice(&temp_buf[..n]);

                    // Try to parse response
                    if let Ok(response) = Response::from_bytes(&buffer) {
                        return Ok(response);
                    }

                    // If buffer gets too large, something is wrong
                    if buffer.len() > 65536 {
                        return Err(anyhow::anyhow!("Response buffer overflow"));
                    }
                }
                Ok(Ok(_)) => {
                    // No data received, continue
                    continue;
                }
                Ok(Err(e)) => {
                    return Err(anyhow::anyhow!("Serial read error: {}", e));
                }
                Err(_) => {
                    return Err(anyhow::anyhow!("Response timeout"));
                }
            }
        }
    }

    pub async fn send_packet_no_ack(&mut self, packet: Packet) -> Result<()> {
        // Send packet without waiting for ACK (for batch operations)
        self.send_packet(&packet).await
    }

    pub async fn send_command(&mut self, packet: Packet) -> Result<Response> {
        // Send packet
        self.send_packet(&packet).await?;

        // Receive response
        let response = self.receive_response().await?;

        // Check response status
        match response.status {
            Status::Success => Ok(response),
            Status::InvalidCommand => Err(anyhow::anyhow!("Invalid command")),
            Status::InvalidAddress => Err(anyhow::anyhow!("Invalid address or size")),
            Status::FlashError => Err(anyhow::anyhow!("Flash operation failed")),
            Status::CrcError => Err(anyhow::anyhow!("CRC error")),
            Status::BufferOverflow => Err(anyhow::anyhow!("Buffer overflow")),
            Status::Timeout => Err(anyhow::anyhow!("Operation timeout")),
            Status::VerificationFailed => Err(anyhow::anyhow!("Data verification failed")),
            Status::Unknown => Err(anyhow::anyhow!("Unknown error")),
        }
    }
}
