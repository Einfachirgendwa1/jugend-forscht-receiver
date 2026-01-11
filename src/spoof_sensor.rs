use package_parser::spoofed_data::SpoofedData;
use package_parser::{DataReceiver, END, MAGIC};
use rand::{rng, Rng};
use std::thread::sleep;
use std::time::Duration;

pub(crate) struct SpoofSensor {
    spoofed_data: SpoofedData,
    create_spoofed_data: Box<dyn FnMut() -> SpoofedData>,
}

impl SpoofSensor {
    pub(crate) fn new(sensor: i32) -> Self {
        let mut time: i32 = 0;
        let mut last_value = rng().random_range(0..200i32);

        let mut create_spoofed_data: Box<dyn FnMut() -> SpoofedData> = Box::new(move || {
            time += 1;
            last_value += rng().random_range(-10..=10i32);
            last_value = last_value.clamp(0, 200);

            SpoofedData::from(&[
                &MAGIC as &[u8],
                &1i32.to_le_bytes(),
                &12i32.to_le_bytes(),
                &time.to_le_bytes(),
                &sensor.to_le_bytes(),
                &last_value.to_le_bytes(),
                &END,
            ] as &[&[u8]])
        });

        Self {
            spoofed_data: create_spoofed_data(),
            create_spoofed_data,
        }
    }
}

unsafe impl Send for SpoofSensor {}
unsafe impl Sync for SpoofSensor {}
impl DataReceiver for SpoofSensor {
    fn get_next_byte(&mut self) -> Option<u8> {
        sleep(Duration::from_millis(50));

        if let x @ Some(_) = self.spoofed_data.get_next_byte() {
            return x;
        }

        self.spoofed_data = (self.create_spoofed_data)();

        self.spoofed_data.get_next_byte()
    }
}
