use axstd::io::{Error, Result, prelude::*};
use axstd::net::{IpAddr, TcpListener, TcpStream};
use core::str::FromStr;
use gdbstub::conn::{Connection, ConnectionExt};
use std::sync::{Arc, Mutex};

const LOCAL_IP: &str = "10.0.2.15";

pub struct GdbServer {
    inner: Arc<Mutex<TcpStream>>,
}

unsafe impl Send for GdbServer {}
unsafe impl Sync for GdbServer {}

impl Connection for GdbServer {
    type Error = Error;

    fn write(&mut self, byte: u8) -> Result<()> {
        let mut stream = self.inner.lock();
        Write::write_all(&mut *stream, &[byte])
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        let mut stream = self.inner.lock();
        Write::write_all(&mut *stream, buf)
    }

    fn flush(&mut self) -> Result<()> {
        let mut stream = self.inner.lock();
        Write::flush(&mut *stream)
    }

    fn on_session_start(&mut self) -> Result<()> {
        Ok(())
    }
}

impl ConnectionExt for GdbServer {
    fn read(&mut self) -> Result<u8> {
        let mut buf = [0u8];
        let mut stream = self.inner.lock();
        match Read::read_exact(&mut *stream, &mut buf) {
            Ok(_) => Ok(buf[0]),
            Err(e) => Err(e),
        }
    }

    fn peek(&mut self) -> Result<Option<u8>> {
        Ok(None)
    }
}

impl GdbServer {
    pub fn new(port: u16) -> Result<Self> {
        let addr = IpAddr::from_str(LOCAL_IP).unwrap();
        let listener = TcpListener::bind((addr, port))?;
        let stream = listener.accept()?.0;
        Ok(Self {
            inner: Arc::new(Mutex::new(stream)),
        })
    }
}
