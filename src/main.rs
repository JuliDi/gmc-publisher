extern crate clap;
extern crate serialport;
extern crate ureq;

use std::io::{self, Write};
use std::time::Duration;

use clap::{App, AppSettings, Arg};
use serialport::prelude::*;

use std::{thread, time};

fn main() {
    let matches = App::new("GMC Logger")
        .about("Reads the current CPM from the GMC's serial port and publishes it to gmcmap.com")
        .setting(AppSettings::DisableVersion)
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .help("The device path to a serial port, e. g. /dev/tty.USB0")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("AID")
                .short("a")
                .long("aid")
                .value_name("AID")
                .help("The gmcmap.com user account ID")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("GID")
                .short("g")
                .long("gid")
                .value_name("GID")
                .help("The gmcmap.com Geiger Counter ID")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let port_name = matches.value_of("port").unwrap();
    let aid = matches.value_of("AID").unwrap();
    let gid = matches.value_of("GID").unwrap();

    let settings = SerialPortSettings {
        baud_rate: 115200,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(50),
    };

    match serialport::open_with_settings(&port_name, &settings) {
        Ok(mut port) => {
            let mut serial_buf: Vec<u8> = vec![0; 1000];

            // Clear port by closing pending commands and reading the response
            match port.write(">>".as_bytes()) {
                Ok(size) => println!("Wrote {} bytes to end all previous commands!", size),
                Err(e) => eprintln!("{:?}", e),
            }
            // Give the device some time to respond
            thread::sleep(time::Duration::from_millis(200));

            match port.clear(ClearBuffer::All) {
                Ok(_) => (),
                Err(e) => eprintln!("Error clearing buffer! Message: {}", e),
            }
            println!("Buffer cleared");

            match port.write(b"<GETCPM>>") {
                Ok(t) => {
                    println!("Wrote {} bytes to get the current CPM", t);
                }
                Err(e) => eprintln!("{:?}", e),
            }

            // Force output before continuing the program
            match port.flush() {
                _ => (),
            }

            // Give the device some time to respond
            thread::sleep(time::Duration::from_millis(200));

            match port.bytes_to_read() {
                Ok(t) if t == 2 => (),
                _ => panic!("Device's reponse doesn't have the correct length of 2!"),
            }
            
            let current_cpm: u16; // Counts Per Minute
            let current_usvph: f32; // uSv/h

            // Read the answer from the serial port
            match port.read(serial_buf.as_mut_slice()) {
                Ok(t) => println!("Result: size: {}, content: {:?}", t, &serial_buf[..t]),
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                    panic!("Reading CPM from port timed out!")
                }
                Err(e) => panic!("{:?}", e),
            }

            // Combine the two u8 values into one u16 value
            current_cpm = ((serial_buf[0] as u16) << 8) | serial_buf[1] as u16;

            // Convert the CPM to uSv/h
            current_usvph = current_cpm as f32 * 6.5 / 1000.0;
            println!("{} CPM, {} uSv/h", current_cpm, current_usvph);

            match publish_result(&aid, &gid, &current_cpm, &current_usvph) {
                Ok(_) => println!("Data successfully published"),
                Err(e) => eprintln!("Publishing data failed! Error: '{}'", e),
            }
        }
        Err(e) => {
            eprintln!("Failed to open port {}. Error: {}", port_name, e);
            ::std::process::exit(1);
        }
    }

    fn publish_result(aid: &str, gid: &str, cpm: &u16, usvph: &f32) -> Result<(), String> {
        // Compose the URL for the GET request
        let request_url = format!(
            "http://www.GMCmap.com/log2.asp?AID={}&GID={}&CPM={}&uSV={}",
            aid, gid, cpm, usvph
        );

        // Perform the get request with timeouts
        let resp = ureq::get(&request_url.as_str())
            .timeout_connect(10_000)
            .timeout_read(2_000)
            .call();
        if resp.ok() {
            // Check whether gmcmap.com returned OK or an error
            match resp.into_string() {
                Ok(t) if t.contains("OK.ERR0") => Ok(()),
                Ok(t) => Err(format!("gmcmap.com returned errorcode {}", t)),
                Err(e) => panic!("Getting string from response failed with error: {}", e),
            }
        } else {
            // Handle other errors like missing internet connection
            Err(format!(
                "http request failed, response stauts: {} - {}",
                resp.status(),
                resp.status_text()
            ))
        }
    }
}
