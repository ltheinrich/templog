mod plotter;

use kern::meta::{init_name, init_version};
use kern::Command;
use plotter::{average, parse_log, plot, sort};
use std::env;
use std::fs::{File, OpenOptions};
use std::io::prelude::{Read, Write};
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static CARGO_TOML: &'static str = include_str!("../Cargo.toml");

fn main() {
    println!(
        "{} {} (c) 2021 Lennart Heinrich",
        init_name(CARGO_TOML),
        init_version(CARGO_TOML)
    );
    let args: Vec<String> = env::args().collect();
    let cmd = Command::from(&args, &["help"]);
    if cmd.option("help") {
        println!(
            "{} --interval 500 --logfile templog.csv --tempfile /sys/devices/platform/nct6775.2592/hwmon/hwmon3/temp7_input",
            cmd.command()
        );
        println!(
            "{} --avg 1 --logfile templog.csv --graph templog.svg",
            cmd.command()
        );
        return;
    }

    let avg = cmd.parameter("avg", 1);
    let interval = cmd.parameter("interval", 500);
    let temp_path = cmd.param(
        "tempfile",
        "/sys/devices/platform/nct6775.2592/hwmon/hwmon3/temp7_input",
    );
    let graph_path = cmd.param("graph", "templog.svg");
    let log_path = cmd.param("logfile", "templog.csv");
    let mut log_file = OpenOptions::new()
        .read(true)
        .create(true)
        .append(true)
        .open(log_path)
        .expect(&format!("Could not open or create log file '{}'", log_path));

    if cmd.option("graph") || cmd.arguments().contains(&"graph") {
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
        loop {
            write_temperature(&mut log_file, get_temperature(&temp_path));
            sleep(Duration::from_millis(interval));
        }
    }
}

fn write_temperature(file: &mut File, temp: f64) {
    let temp_line = format!("{},{}\n", get_time(), temp);
    file.write_all(temp_line.as_bytes())
        .expect("Could not write to log file");
}

fn get_time() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Misconfigured time or are you a time traveller?")
        .as_millis()
}

fn get_temperature(path: &str) -> f64 {
    let mut file = File::open(path).expect(&format!("Could not open temperature file '{}'", path));
    let mut buf = String::with_capacity(6);
    file.read_to_string(&mut buf)
        .expect("Failed to read temperature file");
    let temp_int: i32 = buf
        .strip_suffix('\n')
        .unwrap_or_else(|| &buf)
        .parse()
        .expect("Temperature is not a 32-bit integer");
    temp_int as f64 / 1000.0
}
