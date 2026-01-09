### Package:

- MAGIC: 77u8, 87u8, 100u8
- version: i32
- data_length: i32
- data: [u8; data_length]
- END: 10u8, 10u8, 0u8

#### Package v1:

- version must be 1
- data_length must be 12
- data:

    - timestamp: i32
    - sensor: i32
    - value: i32
