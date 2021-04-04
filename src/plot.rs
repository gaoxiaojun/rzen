use crate::bar::Bar;
use chrono::prelude::*;
use plotly::common::{TickFormatStop, Title};
use plotly::layout::{Axis, RangeSelector, RangeSlider, SelectorButton, SelectorStep, StepMode};
use plotly::{Candlestick, Layout, Ohlc, Plot, Scatter};
use serde::Deserialize;
use std::env;
use std::path::PathBuf;
use std::vec::Vec;

pub fn draw_bar(bars: &Vec<Bar>) {
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
}
