#![allow(dead_code)]

pub trait Codec<T> {
    type Error;
    fn encode(dst: &mut [u8], v: &T) -> Result<usize, Self::Error>;
    fn decode(srd: &[u8]) -> Result<T, Self::Error>;
}

pub struct Json;

impl<T> Codec<T> for Json
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    type Error = serde_json_core::de::Error;

    fn encode(dst: &mut [u8], v: &T) -> Result<usize, Self::Error> {
        let s = serde_json_core::to_slice(v, dst).map(|s| s.len())?;

        Ok(s)
    }

    fn decode(src: &[u8]) -> Result<T, Self::Error> {
        let (v, _rem) = serde_json_core::from_slice(src)?;
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

    fn decode(srd: &[u8]) -> Result<T, Self::Error> {
        postcard::from_bytes(src)
    }
}
