use crate::bar::Bar;
use askama::Template;
use chrono::prelude::*;
use rand::{thread_rng, Rng};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::vec::Vec;

const DEFAULT_HTML_APP_NOT_FOUND: &str = "Could not find default application for HTML files.";

#[derive(Template)]
#[template(path = "data.json", escape = "none")]
struct PlotTemplate<'a> {
    plot_data: &'a str,
}

pub struct Plot {}

impl Plot {
    pub fn new() -> Self {
        Plot {}
    }

    pub fn show(&self) {
        let rendered = self.render();
        let rendered = rendered.as_bytes();
        let mut temp = env::temp_dir();

        let mut plot_name = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(22)
            .collect::<String>();
        plot_name.push_str(".html");
        plot_name = format!("plotly_{}", plot_name);

        temp.push(plot_name);
        let temp_path = temp.to_str().unwrap();
        {
            let mut file = File::create(temp_path).unwrap();
            file.write_all(rendered)
                .expect("failed to write html output");
            file.flush().unwrap();
        }
    }

    /*fn templates_root_path() -> PathBuf {
        let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let templates = root.join("templates");
        templates
    }*/

    fn render_plot_data(&self) -> String {
        let mut plot_data = String::new();
        /*for (idx, trace) in self.traces.iter().enumerate() {
            let s = trace.serialize();
            plot_data.push_str(format!("var trace_{} = {};\n", idx, s).as_str());
        }
        // plot_data.push_str("\n");
        plot_data.push_str("var data = [");
        for idx in 0..self.traces.len() {
            if idx != self.traces.len() - 1 {
                plot_data.push_str(format!("trace_{},", idx).as_str());
            } else {
                plot_data.push_str(format!("trace_{}", idx).as_str());
            }
        }
        plot_data.push_str("];\n");
        let layout_data = match &self.layout {
            Some(layout) => format!("var layout = {};", Trace::serialize(layout)),
            None => {
                let mut s = String::from("var layout = {");
                s.push_str("};");
                s
            }
        };
        plot_data.push_str(layout_data.as_str());*/
        plot_data
    }

    fn render(&self) -> String {
        let plot_data = self.render_plot_data();
        let tmpl = PlotTemplate {
            plot_data: plot_data.as_str(),
        };
        tmpl.render().unwrap()
    }

    pub fn draw_bar(&self, bars: &Vec<Bar>) {}

    #[cfg(target_os = "linux")]
    fn show_with_default_app(temp_path: &str) {
        Command::new("xdg-open")
            .args(&[temp_path])
            .output()
            .expect(DEFAULT_HTML_APP_NOT_FOUND);
    }

    #[cfg(target_os = "macos")]
    fn show_with_default_app(temp_path: &str) {
        Command::new("open")
            .args(&[temp_path])
            .output()
            .expect(DEFAULT_HTML_APP_NOT_FOUND);
    }

    #[cfg(target_os = "windows")]
    fn show_with_default_app(temp_path: &str) {
        Command::new("cmd")
            .arg("/C")
            .arg(format!(r#"start {}"#, temp_path))
            .output()
            .expect(DEFAULT_HTML_APP_NOT_FOUND);
    }
}

/*pub fn draw_bar(bars: &Vec<Bar>) {
    let mut x: Vec<String> = Vec::new();
    let mut open: Vec<f64> = Vec::new();
    let mut high: Vec<f64> = Vec::new();
    let mut low: Vec<f64> = Vec::new();
    let mut close: Vec<f64> = Vec::new();

    for bar in bars {
        let timestamp = bar.time;
        let naive = NaiveDateTime::from_timestamp(timestamp / 1000, 0);
        let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
        let newdate = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
        x.push(newdate);
        open.push(bar.open);
        high.push(bar.high);
        low.push(bar.low);
        close.push(bar.close);
    }

    let trace1 = Candlestick::new(x, open, high, low, close);

    let mut plot = Plot::new();
    let layout = Layout::new().auto_size(true);
    plot.set_layout(layout);
    plot.add_trace(trace1);
    plot.show();
}*/
