mod plotter;

use kern::meta::{init_name, init_version};
use kern::CliBuilder;
use plotter::{average, parse_log, plot, sort};
use std::env;
use std::fs::{File, OpenOptions};
use std::io::prelude::{Read, Write};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{sleep, spawn};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static CARGO_TOML: &str = include_str!("../Cargo.toml");

fn main() {
    println!(
        "{} {} (c) 2021 Lennart Heinrich",
        init_name(CARGO_TOML),
        init_version(CARGO_TOML)
    );
    let args: Vec<String> = env::args().collect();
    let cmd = CliBuilder::new()
        .options(&["help"])
        .paramopts(&["graph"])
        .build(&args);
    if cmd.option("help") {
        println!(
            "{} --buffer 4096 --interval 500 --logfile templog.csv --tempfile /sys/devices/platform/nct6775.2592/hwmon/hwmon3/temp7_input",
            cmd.command()
        );
        println!(
            "{} --avg 1 --logfile templog.csv --graph templog.svg",
            cmd.command()
        );
        return;
    }

    let avg = cmd.parameter("avg", 1);
    let buf_len = cmd.parameter("buffer", 4096);
    let interval = cmd.parameter("interval", 500);
    let temp_path = cmd.param(
        "tempfile",
        "/sys/devices/platform/nct6775.2592/hwmon/hwmon3/temp7_input",
    );
    let graph_path = cmd.param("graph", "templog.svg");
    let log_path = cmd.param("logfile", "templog.csv");
    let log_file = OpenOptions::new()
        .read(true)
        .create(true)
        .append(true)
        .open(log_path)
        .unwrap_or_else(|_| panic!("Could not open or create log file '{}'", log_path));

    if cmd.option("graph") || cmd.parameters().contains_key(&"graph") {
        println!(
            "Creating graph from log file '{}' averaging every {} entrie(s) ...",
            log_path, avg
        );
        let mut log = parse_log(log_path);
        sort(&mut log);
        let averaged = average(&log, avg);
        plot(averaged, graph_path);
        println!("Created graph '{}'", graph_path);
    } else {
        println!(
            "Logging to file '{}' every {} ms from temperature file '{}' ...",
            log_path, interval, temp_path
        );
        let writer = buf_writer(log_file, buf_len);
        loop {
            writer
                .send(get_temperature(&temp_path))
                .expect("Buffered writer channel (sender) crashed");
            sleep(Duration::from_millis(interval));
        }
    }
}

fn get_time() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Misconfigured time or are you a time traveller?")
        .as_millis()
}

fn get_temperature(path: &str) -> f64 {
    let mut file =
        File::open(path).unwrap_or_else(|_| panic!("Could not open temperature file '{}'", path));
    let mut buf = String::with_capacity(6);
    file.read_to_string(&mut buf)
        .expect("Failed to read temperature file");
    let temp_int: i32 = buf
        .strip_suffix('\n')
        .unwrap_or(&buf)
        .parse()
        .expect("Temperature is not a 32-bit integer");
    temp_int as f64 / 1000.0
}

fn buf_writer(mut file: File, buf_len: usize) -> Sender<f64> {
    let (sender, receiver): (Sender<f64>, Receiver<f64>) = channel();
    spawn(move || {
        let mut buf = String::with_capacity(buf_len);
        while let Ok(temp) = receiver.recv() {
            buf.push_str(&get_time().to_string());
            buf.push(',');
            buf.push_str(&temp.to_string());
            buf.push('\n');
            if buf.len() >= buf_len {
                file.write_all(buf.as_bytes())
                    .expect("Could not write to log file");
                buf.clear();
            }
        }
        panic!("Buffered writer channel (receiver) crashed");
    });
    sender
}
