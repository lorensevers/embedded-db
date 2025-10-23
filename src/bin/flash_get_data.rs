// To test this, Run flash_demo.rs and then run this program.
// -> Should show data from flash_demo.rs
// A second test would be to run flash_demo.rs and then erase the flash and run this program.
// -> Should show no data since flash is erased
// A third test would be to run flash_demo.rs and then turn off the board and run this program and then run flash_demo.rs again.
// -> Should still show flash data

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::*;
use embedded_db::{codec::Codec, db::Database, flash::FlashStorage};
use hal::pac;
use nrf52840_hal as hal;

const FLASH_STORAGE_ADDR: u32 = 0x000E_F000;

pub struct U32Codec;

impl Codec<u32> for U32Codec {
    type Error = ();

    fn encode(buffer: &mut [u8], val: &u32) -> Result<usize, Self::Error> {
        if buffer.len() < 4 {
            return Err(());
        }
        buffer[..4].copy_from_slice(&val.to_le_bytes());
        Ok(4)
    }

    fn decode(buffer: &[u8]) -> Result<u32, Self::Error> {
        if buffer.len() < 4 {
            return Err(());
        }
        Ok(u32::from_le_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3],
        ]))
    }
}

#[entry]
fn main() -> ! {
    info!("Flash Test Starting!");

    let p = pac::Peripherals::take().unwrap();

    let mut flash = FlashStorage::new(p.NVMC);

    type MyDb = Database<u32, u32, U32Codec, 16, 256, 4>;

    // Start with an empty database in memory
    let mut db = MyDb::new();

    info!("Attempting to load from flash...");

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

    info!("Database now has {} entries", db.len());

    info!("Complete!");
    embedded_db::idle_forever()
}
