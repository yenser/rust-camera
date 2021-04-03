extern crate v4l;

mod common; 

use common::SOCKET_PATH;

use std::fs;
use std::time::Instant;
use std::net::TcpStream;
use std::io::Write;

use v4l::buffer::Type;
use v4l::io::traits::CaptureStream;
use v4l::prelude::*;
use v4l::video::Capture;
use v4l::FourCC;


fn main() {

    // Determine which device to use
    let source: String = String::from("/dev/video0");
    println!("Using device: {}\n", source);

    // Capture 4 frames by default
    let count = 1;

    // Allocate 4 buffers by default
    let buffers = 4;

    let mut dev = Device::with_path(source).unwrap();

    let mut fmt = dev.format().expect("Failed to read format");
    fmt.width = 1280;
    fmt.height = 720;
    fmt.fourcc = FourCC::new(b"MP42");
    dev.set_format(&fmt).expect("Failed to write format");


    let format = dev.format().unwrap();
    let params = dev.params().unwrap();
    println!("Active format:\n{}", format);
    println!("Active parameters:\n{}", params);

    // Setup a buffer stream and grab a frame, then print its data
    let mut stream = MmapStream::with_buffers(&mut dev, Type::VideoCapture, buffers).unwrap();

    // warmup
    stream.next().unwrap();

    // let mut tcp_stream = TcpStream::connect(SOCKET_PATH).unwrap();

    let start = Instant::now();
    let mut megabytes_ps: f64 = 0.0;

    fs::create_dir_all("./videos/").unwrap();
    
    // tcp_stream.write(&format.size.to_be_bytes()).unwrap();


    for i in 0..count {
        let t0 = Instant::now();
        let (buf, meta) = stream.next().unwrap();
        let duration_us = t0.elapsed().as_micros();

        // tcp_stream.write(buf).unwrap();
        // tcp_stream.write("\0").unwrap();

        fs::write("./videos/video0.mp4", buf).unwrap();
        // fs::write("./videos/video0.mp4", b"\n").unwrap();

        let cur = buf.len() as f64 / 1_048_576.0 * 1_000_000.0 / duration_us as f64;
        if i == 0 {
            megabytes_ps = cur;
        } else {
            // ignore the first measurement
            let prev = megabytes_ps * (i as f64 / (i + 1) as f64);
            let now = cur * (1.0 / (i + 1) as f64);
            megabytes_ps = prev + now;
        }

        println!("Buffer");
        println!("  sequence  : {}", meta.sequence);
        println!("  timestamp : {}", meta.timestamp);
        println!("  flags     : {}", meta.flags);
        println!("  length    : {}", buf.len());
    }

    println!();
    println!("FPS: {}", count as f64 / start.elapsed().as_secs_f64());
    println!("MB/s: {}", megabytes_ps);
}