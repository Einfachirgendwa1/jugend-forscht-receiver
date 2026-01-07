mod tests;

fn main() {}

const MAGIC: [u8; 3] = [77, 87, 100];
const END: [u8; 3] = [10, 10, 0];

trait DataReceiver {
    fn get_next_byte(&mut self) -> Option<u8>;
}

trait DataReceiverExt {
    fn read_next_package(&mut self) -> Option<Package>;
}

impl<T: DataReceiver> DataReceiverExt for T {
    fn read_next_package(&mut self) -> Option<Package> {
        let mut buffer = Vec::new();

        while let Some(byte) = self.get_next_byte() {
            buffer.push(byte);

            if let Some(package) = Package::try_from_buffer(&buffer) {
                return Some(package);
            }
        }

        None
    }
}

#[derive(Debug, PartialEq)]
struct Package {
    version: i32,
    data_len: i32,
    data: Vec<u8>,
}

#[derive(Debug)]
struct PackageV1 {
    timestamp: i32,
    value: i32,
}

const MINIMUM_LENGTH: usize = MAGIC.len() + END.len() + 2 * size_of::<i32>();

impl Package {
    fn try_from_buffer(data: &[u8]) -> Option<Self> {
        eprint!(" > ");
        if data.len() < MINIMUM_LENGTH {
            eprintln!("Package incomplete");
            return None;
        }

        let mut start_idx = 0;
        while !data[start_idx..].starts_with(&MAGIC) {
            start_idx += 1;

            if start_idx >= data.len() {
                eprintln!("No MAGIC");
                return None;
            }
        }

        if !data.ends_with(&END) {
            eprintln!("No END");
            return None;
        }

        if data.len() - start_idx < MINIMUM_LENGTH {
            eprintln!("Package too short");
            return None;
        }

        let start_idx = start_idx + MAGIC.len();
        let end_idx = data.len() - END.len();

        let version = bytes::<i32, 4>(&data[start_idx..], i32::from_le_bytes)?;
        let data_len = bytes::<i32, 4>(&data[start_idx + 4..], i32::from_le_bytes)?;

        if start_idx + 8 + data_len as usize != end_idx {
            eprintln!(
                "Package data_len doesn't match {} vs {}",
                start_idx + 8 + data_len as usize,
                end_idx
            );
            return None;
        }

        let data = data[start_idx + 8..end_idx].to_vec();

        let res = Self {
            version,
            data_len,
            data,
        };

        assert_eq!(res.data_len as usize, res.data.len());

        Some(res)
    }
}

impl PackageV1 {
    fn try_from(package: Package) -> Option<Self> {
        if package.version != 1 {
            return None;
        }

        if package.data_len as usize != size_of::<Self>() {
            eprintln!("Package length doesnt match!");
            return None;
        }

        let timestamp = bytes::<i32, 4>(&package.data[..4], i32::from_le_bytes)?;
        let value = bytes::<i32, 4>(&package.data[4..8], i32::from_le_bytes)?;

        let res = Self { timestamp, value };
        Some(res)
    }
}

fn bytes<T, const N: usize>(bytes: &[u8], f: impl FnOnce([u8; N]) -> T) -> Option<T> {
    let Ok(sized_bytes) = (&bytes[..N]).try_into() else {
        return None;
    };

    Some(f(sized_bytes))
}

struct SpoofedData {
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

impl<const N: usize> From<&[&[u8]; N]> for SpoofedData {
    fn from(value: &[&[u8]; N]) -> Self {
        Self {
            data: value.iter().flat_map(|x| x.iter()).cloned().collect(),
            index: 0,
        }
    }
}
