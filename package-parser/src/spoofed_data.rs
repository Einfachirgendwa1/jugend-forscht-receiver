use crate::DataReceiver;

pub struct SpoofedData {
    data: Vec<u8>,
    index: usize,
}

impl DataReceiver for SpoofedData {
    fn get_next_byte(&mut self) -> Option<u8> {
        if self.index >= self.data.len() {
            return None;
        }

        let byte = self.data[self.index];
        self.index += 1;
        Some(byte)
    }
}

impl From<&[u8]> for SpoofedData {
    fn from(value: &[u8]) -> Self {
        Self {
            data: value.to_vec(),
            index: 0,
        }
    }
}

impl From<&[&[u8]]> for SpoofedData {
    fn from(value: &[&[u8]]) -> Self {
        Self::from(
            value
                .iter()
                .flat_map(|x| x.to_vec())
                .collect::<Vec<u8>>()
                .as_slice(),
        )
    }
}
