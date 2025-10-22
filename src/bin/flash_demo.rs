#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::*;
use embedded_db::{codec::Codec, db::Database, flash::FlashStorage};
use hal::pac;
use nrf52840_hal as hal;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum EmbeddedError {
    BufferTooSmall,
    Other,
}

pub struct U32Codec;

impl Codec<u32> for U32Codec {
    type Error = EmbeddedError;

    fn encode(buffer: &mut [u8], val: &u32) -> Result<usize, Self::Error> {
        if buffer.len() < 4 {
            return Err(EmbeddedError::BufferTooSmall);
        }
        buffer[..4].copy_from_slice(&val.to_le_bytes());
        Ok(4)
    }

    fn decode(buffer: &[u8]) -> Result<u32, Self::Error> {
        if buffer.len() < 4 {
            return Err(EmbeddedError::Other);
        }
        Ok(u32::from_le_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3],
        ]))
    }
}

// Flash storage address - using 64KB
// Using 0x000E_F000 to 0x000F_F000 (64KB)
// This should be almost the last 64KB of flash if my math is correct
// https://docs.nordicsemi.com/bundle/ps_nrf52840/page/memory.html
const FLASH_STORAGE_ADDR: u32 = 0x000E_F000;

#[entry]
fn main() -> ! {
    info!("Flash Test Starting!");

    let p = pac::Peripherals::take().unwrap();

    let mut flash = FlashStorage::new(p.NVMC);

    type MyDb = Database<u32, u32, U32Codec, 16, 256, 4>;

    // Start with an empty database in memory
    let mut db = MyDb::new();

    info!("Attempting to load from flash...");

    // Try to get data from flash, and load it into the database that is in memory
    match db.load_from_flash(&mut flash, FLASH_STORAGE_ADDR) {
        Ok(_) => {
            info!("Loaded {} entries from flash", db.len());

            // Print what we loaded
            for i in 0..16 {
                if let Ok(Some(val)) = db.get_uncached(&i) {
                    info!("  Key {}: Value {}", i, val);
                }
            }
        }
        Err(e) => {
            info!("No existing data or error loading: {:?}", e);
        }
    }

    info!("Adding new data...");

    // Add or update some entries (in memory)
    let _ = db.put(1, 100);
    let _ = db.put(2, 200);
    let _ = db.put(3, 300);

    // Increment existing values
    if let Ok(Some(val)) = db.get(&1) {
        let _ = db.put(1, val + 1);
    }

    info!("Database now has {} entries", db.len());

    // Save to flash
    info!("Saving to flash...");
    match db.save_to_flash(&mut flash, FLASH_STORAGE_ADDR) {
        Ok(_) => {
            info!("Successfully saved to flash!");
            info!("If you turn offf the device it will still have the data (in flash)");
        }
        Err(e) => {
            error!("Failed to save to flash: {:?}", e);
        }
    }

    // Display final state of the in memory database
    info!("Final database contents:");
    for i in 0..16 {
        if let Ok(Some(val)) = db.get_uncached(&i) {
            info!("  Key {}: Value {}", i, val);
        }
    }

    info!("Complete!");
    embedded_db::idle_forever()
}
