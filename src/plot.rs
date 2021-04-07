use crate::bar::Bar;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::vec::Vec;
use std::{env, fs::File};

const DEFAULT_HTML_APP_NOT_FOUND: &str = "Could not find default application for HTML files.";

fn templates_root_path() -> PathBuf {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let templates = root.join("templates");
    templates
}

fn render_bar_vue(bar: &Bar) -> String {
    format!(
        "[ {}, {}, {}, {}, {} ]",
        bar.time, bar.open, bar.high, bar.low, bar.close
    )
}

fn render_bars_vue(bars: &Vec<Bar>) -> String {
    let mut buf = String::new();
    let header = r#"Data = {
        "chart": {
            "type": "Candles",
            "data": [
                    "#;
    let bottom = r#"
            ],
            "indexBased": true
        }
    }"#;
    buf.push_str(header);
    let data: Vec<String> = bars.into_iter().map(|bar| render_bar_vue(bar)).collect();
    let all_data = data.join(",\n");
    buf.push_str(all_data.as_str());
    buf.push_str(bottom);
    buf
}

pub fn draw_bar_vue(bars: &Vec<Bar>) {
    let rendered = render_bars_vue(bars);
    let rendered = rendered.as_bytes();
    let mut temp = env::temp_dir();
    let mut src = templates_root_path();

    // write data.json
    temp.push("data.json");
    let temp_path = temp.to_str().unwrap();
    {
        let mut file = File::create(temp_path).unwrap();
        file.write_all(rendered)
            .expect("failed to write html output");
        file.flush().unwrap();
    }
    println!("{}", temp.display());
    temp.pop();

    // copy index.html
    temp.push("index_vue.html");
    src.push("index.html");
    std::fs::copy(src.as_path(), temp.as_path()).expect("failed to copy index.html");

    // display in browser
    show_with_default_app(temp.to_str().unwrap());
}

fn render_bar_tradingview(bar: &Bar) -> String {
    format!(
        "{{ time:{}, open:{}, high:{}, low:{}, close:{} }}",
        bar.time / 1000,
        bar.open,
        bar.high,
        bar.low,
        bar.close
    )
}

fn render_bars_tradingview(bars: &Vec<Bar>) -> String {
    let mut buf = String::new();
    let header = "Data = [\n";
    let bottom = "]";
    buf.push_str(header);
    let data: Vec<String> = bars
        .into_iter()
        .map(|bar| render_bar_tradingview(bar))
        .collect();
    let all_data = data.join(",\n");
    buf.push_str(all_data.as_str());
    buf.push_str(bottom);
    buf
}

pub fn draw_bar_tradingview(bars: &Vec<Bar>) {
    let rendered = render_bars_tradingview(bars);
    let rendered = rendered.as_bytes();
    let mut temp = env::temp_dir();
    let mut src = templates_root_path();

    // write data.json
    temp.push("chart-data.json");
    let temp_path = temp.to_str().unwrap();
    {
        let mut file = File::create(temp_path).unwrap();
        file.write_all(rendered)
            .expect("failed to write html output");
        file.flush().unwrap();
    }
    println!("{}", temp.display());
    temp.pop();

    // copy index.html
    temp.push("index_tradingview.html");
    src.push("index_chart.html");
    std::fs::copy(src.as_path(), temp.as_path()).expect("failed to copy index.html");

    // display in browser
    show_with_default_app(temp.to_str().unwrap());
}

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
