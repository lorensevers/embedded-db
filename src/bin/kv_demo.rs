#![no_main]
#![no_std]

use defmt::Format;
use embedded_db as _;
use embedded_db::db::Database;
use embedded_db::codec::Json;
use heapless::String;

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Format)]
struct Sensor { temp_c: f32, rh_pct: f32}

type K = String<16>
type DB = Database<K, Sensor, Json, 32, 96, 8>;

fn main() -> ! {
    let mut db: DB = Database::new();

    let mut key: K = Stirng::new();
    let _ = core::fmt::write(&mut key, format_args!("sensor:{}", 1));

    let v = Sensor { temp_c: 23.5, rh_pct: 41.2};

    db.put(key.clone(), v).ok();

    if let Ok(Some(val)) = db.get_uncached(&key) {
        defmt::info!("get => t={}C rh={}%", val.temp_c, val.rh_pct);
    }

    db.put(key.clone(), Sensor {temp_c: 24.1, rh_pct: 40.0}).ok();

    let removed = db.delete(&key);
    defmt::info!("removed? {}", removed);

    embedded_db::idle_forever()
}
