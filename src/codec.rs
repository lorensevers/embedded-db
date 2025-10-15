// This Codec allows us to encode and decode data
// Currently we support JSON and Postcard
// Postcard will store the data in a more compact format

#![allow(dead_code)]

pub trait Codec<T> {
    type Error;
    fn encode(dst: &mut [u8], v: &T) -> Result<usize, Self::Error>;
    fn decode(src: &[u8]) -> Result<T, Self::Error>;
}

pub enum JsonError {
    Ser(serde_json_core::ser::Error),
    De(serde_json_core::de::Error),
}
impl From<serde_json_core::ser::Error> for JsonError {
    fn from(e: serde_json_core::ser::Error) -> Self {
        JsonError::Ser(e)
    }
}
impl From<serde_json_core::de::Error> for JsonError {
    fn from(e: serde_json_core::de::Error) -> Self {
        JsonError::De(e)
    }
}

pub struct Json;
impl<T> Codec<T> for Json
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    type Error = JsonError;

    fn encode(dst: &mut [u8], v: &T) -> Result<usize, Self::Error> {
        let n = serde_json_core::to_slice(v, dst).map_err(JsonError::from)?;
        Ok(n)
    }

    fn decode(src: &[u8]) -> Result<T, Self::Error> {
        let (v, _rem) = serde_json_core::from_slice(src).map_err(JsonError::from)?;
        Ok(v)
    }
}

pub struct Postcard;
impl<T> Codec<T> for Postcard
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    type Error = postcard::Error;

    fn encode(dst: &mut [u8], v: &T) -> Result<usize, Self::Error> {
        Ok(postcard::to_slice(v, dst)?.len())
    }

    fn decode(src: &[u8]) -> Result<T, Self::Error> {
        postcard::from_bytes(src)
    }
}
