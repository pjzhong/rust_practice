use crate::chart::Chart;

#[derive(Debug)]
pub struct YAxis {}

impl YAxis {
    pub const CHAR_PRECISION: i64 = 6;
    pub const DEC_PRECISION: i64 = 2;
    pub const MARGIN_RIGHT: i64 = 4;

    pub const WIDTH: i64 = YAxis::CHAR_PRECISION
        + YAxis::MARGIN_RIGHT
        + 1
        + YAxis::DEC_PRECISION
        + YAxis::MARGIN_RIGHT;

    pub fn price_to_height(price: f64, min_price: f64, max_price: f64, height: i64) -> f64 {
        (price - min_price) / (max_price - min_price) * height as f64
    }

    pub fn render_line(y: u16, chart: &Chart) -> String {
        match y % 4 {
            0 => YAxis::render_tick(y, chart),
            _ => YAxis::render_empty(),
        }
    }

    fn render_tick(y: u16, chart: &Chart) -> String {
        let min_price = chart.candle_static.min_price;
        let max_price = chart.candle_static.max_price;
        let height = chart.terminal_size.1;

        let price = min_price + (y as f64 * (max_price - min_price) / height as f64);
        let cell_min_length = (YAxis::CHAR_PRECISION + YAxis::DEC_PRECISION + 1) as usize;

        format!(
            "{0:<cell_min_length$.2} |┈{margin}",
            price,
            cell_min_length = cell_min_length,
            margin = " ".repeat(YAxis::MARGIN_RIGHT as usize)
        )
    }

    fn render_empty() -> String {
        let cell = " ".repeat((YAxis::CHAR_PRECISION + YAxis::DEC_PRECISION + 2) as usize);
        let margin = " ".repeat((YAxis::MARGIN_RIGHT + 1) as usize);

        format!("{}|{}", cell, margin)
    }
}
