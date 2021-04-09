#[cfg(test)]
pub mod tests {
    use crate::bar::Bar;
    use crate::time::Time;
    use chrono::prelude::*;
    use csv;

    #[allow(dead_code)]
    pub fn load_eurusd_7_days() -> Vec<Bar> {
        let csv = include_str!("../tests/EURUSD-2021_04_01-2021_04_06.csv");
        load_datetime_bar(&csv)
    }

    #[allow(dead_code)]
    pub fn load_eurusd_10000() -> Vec<Bar> {
        let csv = include_str!("../tests/eurusd_10000.csv");
        load_timestamp_bar(&csv)
    }

    #[allow(dead_code)]
    pub fn load_datetime_bar(csv: &str) -> Vec<Bar> {
        // duka download datetime timezone is GMT+8
        let mut bars: Vec<Bar> = Vec::new();
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(csv.as_bytes());
        let china_timezone = FixedOffset::east(8 * 3600);
        for record in reader.records() {
            let record = record.unwrap();
            let timestr: &str = AsRef::<str>::as_ref(&record[0]);
            let dt = NaiveDateTime::parse_from_str(timestr, "%Y-%m-%d %H:%M:%S").unwrap();
            let china_dt = dt + china_timezone;
            let datetime: DateTime<Utc> = DateTime::from_utc(china_dt, Utc);
            let time = datetime.timestamp_millis();
            let open = AsRef::<str>::as_ref(&record[1]).parse::<f64>().unwrap();
            let close = AsRef::<str>::as_ref(&record[2]).parse::<f64>().unwrap();
            let high = AsRef::<str>::as_ref(&record[3]).parse::<f64>().unwrap();
            let low = AsRef::<str>::as_ref(&record[4]).parse::<f64>().unwrap();
            let bar = Bar::new(time, open, high, low, close);
            bars.push(bar);
        }
        bars
    }

    #[allow(dead_code)]
    pub fn load_timestamp_bar(csv: &str) -> Vec<Bar> {
        let mut bars: Vec<Bar> = Vec::new();
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(csv.as_bytes());

        for record in reader.records() {
            let record = record.unwrap();
            let time: Time = AsRef::<str>::as_ref(&record[0]).parse::<i64>().unwrap();
            let open = AsRef::<str>::as_ref(&record[1]).parse::<f64>().unwrap();
            let high = AsRef::<str>::as_ref(&record[2]).parse::<f64>().unwrap();
            let low = AsRef::<str>::as_ref(&record[3]).parse::<f64>().unwrap();
            let close = AsRef::<str>::as_ref(&record[4]).parse::<f64>().unwrap();
            let bar = Bar::new(time, open, high, low, close);
            bars.push(bar);
        }
        bars
    }
}
