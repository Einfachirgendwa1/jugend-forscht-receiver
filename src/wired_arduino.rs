use package_parser::DataReceiver;
use serialport::SerialPort;
use std::collections::VecDeque;
use std::io::ErrorKind::TimedOut;
use std::io::Read;

pub struct WiredArduino {
    port: Box<dyn SerialPort>,
    backlog: VecDeque<u8>,
}

impl WiredArduino {
    pub fn new(port: Box<dyn SerialPort>) -> Self {
        WiredArduino {
            port,
            backlog: VecDeque::new(),
        }
    }
}

unsafe impl Send for WiredArduino {}
unsafe impl Sync for WiredArduino {}
impl DataReceiver for WiredArduino {
    fn get_next_byte(&mut self) -> Option<u8> {
        loop {
            let mut buffer = [0u8; 1024];

            match self.port.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    self.backlog = VecDeque::from(buffer[..n].to_vec());
                    break;
                }
                Ok(_) => {}
                Err(ref err) if err.kind() == TimedOut => {}
                Err(err) => panic!("failed to read from port: {err}"),
            }
        }

        self.backlog.pop_front()
    }
}
