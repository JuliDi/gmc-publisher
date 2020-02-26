extern crate clap;
extern crate serialport;
extern crate ureq;

use std::io::{self, Write};
use std::time::Duration;

use clap::{App, AppSettings, Arg};
use serialport::prelude::*;

use std::{thread, time};

fn main() {
    let matches = App::new("Serialport Example - Receive Data")
        .about("Reads data from a serial port and echoes it to stdout")
        .setting(AppSettings::DisableVersion)
        .arg(
            Arg::with_name("port")
                .help("The device path to a serial port")
                .use_delimiter(false)
                .required(true),
        )
        .get_matches();
        
    let port_name = matches.value_of("port").unwrap();

    let settings = SerialPortSettings {
    	baud_rate: 115200,
    	data_bits: DataBits::Eight,
    	flow_control: FlowControl::None,
    	parity: Parity::None,
    	stop_bits: StopBits::One,
    	timeout: Duration::from_millis(50),
    };

	println!("{:#?}", settings);

    match serialport::open_with_settings(&port_name, &settings) {
        Ok(mut port) => {
            let mut serial_buf: Vec<u8> = vec![0; 1000];
            let mut serial_buf_string = String::new();
            println!("Receiving data on {} at {} baud:", &port_name, &settings.baud_rate);


            match port.write(">>".as_bytes()) {
            	Ok(size) => println!("Wrote {} bytes!", size),
            	_ => (),
            }
            loop {
                match port.read_to_string(&mut serial_buf_string) {
                    Ok(t) if t == 0 => {println!("no more buffers to read!"); break;},
                    Ok(t) => println!("read {} buffers!", t),
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {println!("Timed out!"); break;},
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            match port.clear(ClearBuffer::All){
            	Ok(_) => (),
            	Err(e) => eprintln!("Error clearing buffer! Message: {}", e),
            }
            println!("Buffer cleared");

            match port.write(b"<GETCPM>>") {
                    Ok(t) => {
                        println!("Wrote {} bytes", t);
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }
            match port.flush(){
            	_ => (),
            }
            thread::sleep(time::Duration::from_secs(1));

            match port.bytes_to_read() {
            	Ok(t) => println!("{} bytes to read", t),
            	_ => (),
            }
            
            let current_cpm: u16;
            let current_usvph: f32;
            match port.read(serial_buf.as_mut_slice()) {
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {println!("Timed out! Result: {:#?}", serial_buf);},
                Err(e) => eprintln!("{:?}", e),
                Ok(t) => {
                	println!("Result: size: {}, content: {:?}", t, &serial_buf[..t]);
                	if t == 2 {
           				current_cpm = ((serial_buf[0] as u16) << 8) | serial_buf[1] as u16;
           				current_usvph = current_cpm as f32 * 6.5 / 1000.0;
           				println!("{} CPM, {} uSv/h", current_cpm, current_usvph);
           				let request_url = format!("http://www.GMCmap.com/log2.asp?AID={}&GID={}&CPM={}&uSV={}", "02834", "59046675878", current_cpm, current_usvph);
           				let resp = ureq::get(&request_url[..]).call();
           				if resp.ok() {
           					println!("{}", resp.into_string().unwrap());
						}
           				
           			}
           		}
           	}

           
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            ::std::process::exit(1);
        }
    }
}