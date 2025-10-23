// Flash storage module for nRF52840
// Uses internal flash memory for persistent storage

// Using embedded_storage
use embedded_storage::nor_flash::{
    ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};

// I believe for other chips there are other hal crates (stm32-hal, esp-hal, etc.)
// Need to do more research on this.
use nrf52840_hal::pac::NVMC;

/// Size of a flash page on nRF52840 (4KB)
/// https://docs.nordicsemi.com/bundle/ps_nrf52840/page/memory.html
/// Pages go from 0 - 255 (256 pages * 4KB = 1MB)
pub const PAGE_SIZE: usize = 4096;
pub const WRITE_ALIGNMENT: u32 = 4;
pub struct FlashStorage {
    nvmc: NVMC,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlashError {
    OutOfBounds,
    Unaligned,
    Other,
}

impl NorFlashError for FlashError {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            FlashError::OutOfBounds => NorFlashErrorKind::OutOfBounds,
            FlashError::Unaligned => NorFlashErrorKind::NotAligned,
            FlashError::Other => NorFlashErrorKind::Other,
        }
    }
}

impl FlashStorage {
    pub fn new(nvmc: NVMC) -> Self {
        Self { nvmc }
    }

    /// Erase a page of flash memory
    /// On Nordic Nrf chips you have to erase a page at a time
    /// I will need to do more research how this works on other chips.
    fn erase_page(&mut self, page_addr: u32) -> Result<(), FlashError> {
        // Page address must start on a page boundary
        if page_addr % PAGE_SIZE as u32 != 0 {
            return Err(FlashError::OutOfBounds);
        }

        self.nvmc.config.write(|w| w.wen().een());

        // Wait until the flash is ready
        while self.nvmc.ready.read().ready().is_busy() {}

        self.nvmc
            .erasepage()
            .write(|w| unsafe { w.bits(page_addr) });

        // Wait for erase to complete
        while self.nvmc.ready.read().ready().is_busy() {}

        // Disable erase
        self.nvmc.config.write(|w| w.wen().ren());

        Ok(())
    }

    /// Write data to flash
    /// Offset must be word-aligned (4 bytes) and the flash must be erased first
    fn write_bytes(&mut self, offset: u32, data: &[u8]) -> Result<(), FlashError> {
        if offset % WRITE_ALIGNMENT != 0 {
            return Err(FlashError::OutOfBounds);
        }

        // Enable write
        self.nvmc.config.write(|w| w.wen().wen());

        // Wait for ready
        while self.nvmc.ready.read().ready().is_busy() {}

        // Convert bytes to words and write
        let flash_ptr = offset as *mut u32;
        let mut word_offset = 0;

        // Write full words
        while word_offset + 4 <= data.len() {
            let word = u32::from_le_bytes([
                data[word_offset],
                data[word_offset + 1],
                data[word_offset + 2],
                data[word_offset + 3],
            ]);

            // write_volatile is unsafe because we are just moving to a point in memory and writing to it.
            // We could move to a pointer (using our math above) that doesn't exist and write to it.
            unsafe {
                flash_ptr.add(word_offset / 4).write_volatile(word);
            }

            // Wait for write to complete
            while self.nvmc.ready.read().ready().is_busy() {}

            word_offset += 4;
        }

        // Handle remaining bytes (pad with 0xFF)
        if word_offset < data.len() {
            let mut word = 0xFFFFFFFF;

            let position = data.len() - word_offset;

            assert!(position < 4);
            for i in 0..(position) {
                let shift = i * 8;
                word = (word & !(0xFF << shift)) | ((data[word_offset + i] as u32) << shift);
            }

            // write_volatile is unsafe because we are just moving to a point in memory and writing to it.
            unsafe {
                flash_ptr.add(word_offset / 4).write_volatile(word);
            }

            while self.nvmc.ready.read().ready().is_busy() {}
        }

        // Disable write
        self.nvmc.config.write(|w| w.wen().ren());

        Ok(())
    }

    /// Read data from flash to buffer (RAM)
    fn read_bytes(&self, offset: u32, buffer: &mut [u8]) -> Result<(), FlashError> {
        let flash_ptr = offset as *const u8;

        // copy_nonoverlapping is unsafe because we are copying from flash memory to buffer (RAM)
        // but there might not be enough space in the buffer to copy the data.
        // I am not sure if this is the best way to do this.
        unsafe {
            core::ptr::copy_nonoverlapping(flash_ptr, buffer.as_mut_ptr(), buffer.len());
        }

        Ok(())
    }
}

impl ErrorType for FlashStorage {
    type Error = FlashError;
}

impl ReadNorFlash for FlashStorage {
    const READ_SIZE: usize = 1;

    // This calls the read_bytes function to satisfy the ReadNorFlash trait
    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.read_bytes(offset, bytes)
    }

    fn capacity(&self) -> usize {
        // nRF52840 has 1MB flash, but we'll use last 64KB for storage
        64 * 1024
    }
}

impl NorFlash for FlashStorage {
    const WRITE_SIZE: usize = 4; // Must write in 4-byte words
    const ERASE_SIZE: usize = PAGE_SIZE;

    // Same as read function above, we are just calling the erase_page function to satisfy the NorFlash trait
    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        if from % PAGE_SIZE as u32 != 0 || to % PAGE_SIZE as u32 != 0 {
            return Err(FlashError::Other);
        }

        for page_addr in (from..to).step_by(PAGE_SIZE) {
            self.erase_page(page_addr)?;
        }

        Ok(())
    }

    // Same as read/erase function above, we are just calling the write_bytes function to satisfy the NorFlash trait
    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        self.write_bytes(offset, bytes)
    }
}
