#![no_main]

use libfuzzer_sys::fuzz_target;
use package_parser::spoofed_data::SpoofedData;
use package_parser::{DataReceiverExt, PackageV1};

fuzz_target!(|data: &[u8]| {
    let mut spoofed = SpoofedData::from(data);

    let res = spoofed.read_next_package();
    if let Some(ok) = res {
        let _ = PackageV1::try_from(ok, false);
    }
});
