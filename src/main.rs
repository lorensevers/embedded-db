fn main() {
    // Keeping this in for testing for now
    // CAP=128 items, max key length 32 bytes, max value length 256 bytes
    let mut db: embedded_db::EmbeddedDatabase<128, 32, 256> = embedded_db::EmbeddedDatabase::new();

    db.put("device_id", b"alpha-01").unwrap();
    db.put("mode", b"sleep").unwrap();
    db.put("temp", b"1200").unwrap();
    db.put("temp2", b"1200").unwrap();
    db.put("temp3", b"1200").unwrap();

    // This should overwrite the previous value
    db.put("mode", b"active").unwrap();

    // read the value
    if let Some(val) = db.get("mode") {
        println!("mode = {}", core::str::from_utf8(val).unwrap());
    }

    // update if present, this should return and print true
    let is_updated = db.update("mode", b"low_power").unwrap();
    println!("is_updated = {is_updated}");

    // delete the key
    let removed = db.delete("device_id");
    println!("removed device_id? {removed}");

    // print the number of items and capacity
    println!("items: {}", db.len());
    println!("capa: {}", db.capacity());
}
