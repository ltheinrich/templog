use plotlib::view::ContinuousView;
use plotlib::{page::Page, style::LineStyle};
use plotlib::{repr::Plot, style::LineJoin};
use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::Read;

pub fn parse_log(path: &str) -> Vec<(f64, f64)> {
    let mut file = File::open(path).expect(&format!("Could not open log file '{}'", path));
    let mut buf = String::with_capacity(
        file.metadata()
            .expect("Log file has no metadata?")
            .len()
            .try_into()
            .expect("Log file to big! What are you doing?"),
    );
    file.read_to_string(&mut buf)
        .expect("Could not read log file");
    let mut log = Vec::new();
    let mut count = 0u8;
    buf.split('\n').for_each(|line| {
        if !line.is_empty() {
            line.split(',').for_each(|item| match count {
                0 => {
                    log.push((
                        item.parse()
                            .expect("Time can not be parsed to 64-bit float"),
                        0.0,
                    ));
                    count = 1;
                }
                1 => {
                    log.last_mut()
                        .expect("Are you a magician or have you manipulated my tasty memory?")
                        .1 = item.parse().expect("Temperature is not a 64-bit float");
                    count = 2;
                }
                _ => {
                    count = 3;
                    panic!("Log file has wrong format")
                }
            });
            if count == 2 {
                count = 0;
            } else {
                panic!("Log file has wrong format")
            }
        }
    });
    log
}

pub fn sort(data: &mut [(f64, f64)]) {
    data.sort_by(|x, y| x.partial_cmp(y).expect("Can not compare numbers?"));
}

pub fn average(data: &[(f64, f64)], n: usize) -> Vec<(f64, f64)> {
    let len = data.len() / n;
    let mut average_data = Vec::with_capacity(len);
    let first = data.first().expect("No data to average? ...").0;
    for i in 0..len {
        let mut cumulative_x = 0f64;
        let mut cumulative_y = 0f64;
        for j in 0..n {
            let entry = data
                .get(i * n + j)
                .expect("This should not happen, report me. I'm a bug.");
            cumulative_x += entry.0 - first;
            cumulative_y += entry.1;
        }
        average_data.push((cumulative_x / n as f64, cumulative_y / n as f64))
    }
    average_data
}

pub fn plot(data: Vec<(f64, f64)>, graph_path: &str) {
    let plot =
        Plot::new(data).line_style(LineStyle::new().linejoin(LineJoin::Round).colour("#DD3355"));
    let view = ContinuousView::new()
        .add(plot)
        .x_label("Time [ms]")
        .y_label("Temperature [°C]");
    Page::single(&view).save(graph_path).unwrap();
}
