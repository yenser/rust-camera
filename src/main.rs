extern crate v4l;

mod common;

use common::SOCKET_PATH;

use std::io::Error;
use std::io::Write;
use std::net::TcpStream;
use std::time::Instant;
use std::{thread, time};

use std::time::Duration;
use v4l::buffer::Type;
use v4l::io::traits::CaptureStream;
use v4l::prelude::*;
use v4l::video::Capture;
use v4l::FourCC;

fn get_device(name: &str) -> Result<v4l::Device, Error> {
    let source: String = String::from(name);
    println!("Using device: {}\n", source);

    return Device::with_path(source);
}

fn get_camera_stream() -> Result<(v4l::prelude::MmapStream<'static>, v4l::Format), Error> {
    // Determine which device to use

    // Capture 4 frames by default
    // let count = 100;

    // Allocate 4 buffers by default
    let buffers = 4;

    let mut dev = get_device("/dev/video0")?;

    let mut fmt = dev.format().expect("Failed to read format");
    fmt.width = 1280;
    fmt.height = 720;
    fmt.fourcc = FourCC::new(b"JPEG");
    dev.set_format(&fmt).expect("Failed to write format");

    let format = dev.format()?;
    let params = dev.params()?;
    println!("Active format:\n{}", format);
    println!("Active parameters:\n{}", params);

    // Setup a buffer stream and grab a frame, then print its data
    let mut stream = MmapStream::with_buffers(&mut dev, Type::VideoCapture, buffers)?;
    stream.next()?;

    Ok((stream, format))
}

fn connect(
    stream: &mut v4l::prelude::MmapStream<'static>,
    format: &v4l::Format,
) -> Result<(), Error> {
    // warmup

    let timeout = Duration::from_secs(5);
    let mut tcp_stream =
        TcpStream::connect_timeout(&SOCKET_PATH.parse().unwrap(), timeout)?;

    let start = Instant::now();
    let mut megabytes_ps: f64 = 0.0;

    tcp_stream.write(&format.size.to_be_bytes())?;

    loop {
        let t0 = Instant::now();
        let (buf, meta) = stream.next()?;
        let duration_us = t0.elapsed().as_micros();

        tcp_stream.write(buf)?;

        let cur = buf.len() as f64 / 1_048_576.0 * 1_000_000.0 / duration_us as f64;

        let i = meta.sequence;

        if i == 0 {
            megabytes_ps = cur;
        } else {
            // ignore the first measurement
            let prev = megabytes_ps * (i as f64 / (i + 1) as f64);
            let now = cur * (1.0 / (i + 1) as f64);
            megabytes_ps = prev + now;
        }

        
        
        if i % 20 == 0 {
            println!("Buffer");
            println!("  sequence  : {}", meta.sequence);
            println!("  timestamp : {}", meta.timestamp);
            println!("  flags     : {}", meta.flags);
            println!("  length    : {}", buf.len());
            println!();
            println!("FPS: {}", meta.sequence as f64 / start.elapsed().as_secs_f64());
            println!("MB/s: {}", megabytes_ps);
        }
    }

}

fn main() {
    let sleep_time = time::Duration::from_millis(5000);

    let (mut stream, format) = get_camera_stream().unwrap();

    loop {
        match connect(&mut stream, &format) {
            Ok(_) => {},
            Err(err) => {
                println!("Error: {}", err);
                println!("Waiting {} milliseconds", sleep_time.as_millis());
                thread::sleep(sleep_time);
            }
        }

    }
}
