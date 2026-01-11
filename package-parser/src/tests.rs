#[cfg(test)]
mod test_mod {
    use crate::spoofed_data::SpoofedData;
    use crate::{DataReceiverExt, PackageV1, END, MAGIC};

    #[test]
    fn basic() {
        let mut spoofed = SpoofedData::from(&[
            &3i32.to_le_bytes(),
            &394i32.to_le_bytes(),
            &MAGIC as &[u8],
            &1i32.to_le_bytes(),
            &12i32.to_le_bytes(),
            &42i32.to_le_bytes(),
            &17i32.to_le_bytes(),
            &7i32.to_le_bytes(),
            &END,
            &394i32.to_le_bytes(),
            &3i32.to_le_bytes(),
        ] as &[&[u8]]);

        let package = spoofed
            .read_next_package()
            .expect("Failed to parse package");

        let package_v1 = PackageV1::try_from(package, false).expect("Failed to parse package v1");

        assert_eq!(package_v1.timestamp, 42);
        assert_eq!(package_v1.sensor, 17);
        assert_eq!(package_v1.value, 7);
    }

    #[test]
    fn no_magic() {
        let mut spoofed = SpoofedData::from(&[
            &1i32.to_le_bytes(),
            &8i32.to_le_bytes(),
            &42i32.to_le_bytes(),
            &7i32.to_le_bytes(),
            &END as &[u8],
        ] as &[&[u8]]);

        assert_eq!(spoofed.read_next_package(), None);
    }

    #[test]
    fn no_end() {
        let mut spoofed = SpoofedData::from(&[
            &MAGIC as &[u8],
            &1i32.to_le_bytes(),
            &8i32.to_le_bytes(),
            &42i32.to_le_bytes(),
            &7i32.to_le_bytes(),
        ] as &[&[u8]]);

        assert_eq!(spoofed.read_next_package(), None);
    }
}
