// USB CDC utilities and helpers

use defmt::*;
use embassy_usb::class::cdc_acm::CdcAcmClass;
use embassy_usb::driver::EndpointError;

pub trait CdcAcmExt {
    async fn read_packet(&mut self, buffer: &mut [u8]) -> Result<usize, EndpointError>;
    async fn write_packet(&mut self, data: &[u8]) -> Result<(), EndpointError>;
}

impl<'d, D: embassy_usb::driver::Driver<'d>> CdcAcmExt for CdcAcmClass<'d, D> {
    async fn read_packet(&mut self, buffer: &mut [u8]) -> Result<usize, EndpointError> {
        self.read_packet(buffer).await
    }

    async fn write_packet(&mut self, data: &[u8]) -> Result<(), EndpointError> {
        let mut remaining = data;
        
        while !remaining.is_empty() {
            let chunk_size = core::cmp::min(remaining.len(), 64); // USB full-speed max packet size
            let chunk = &remaining[..chunk_size];
            
            self.write_packet(chunk).await?;
            remaining = &remaining[chunk_size..];
        }
        
        Ok(())
    }
}
