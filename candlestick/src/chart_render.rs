use colored::{control, Colorize};

use crate::candle_set::{Candle, CandleStatistics, CandleType};
use crate::chart::Chart;
use crate::yaxis::YAxis;

#[derive(Debug)]
pub struct ChartRender {
    pub bearish_color: (u8, u8, u8),
    pub bullish_color: (u8, u8, u8),
}

impl ChartRender {
    const UNICODE_VOID: char = ' ';
    const UNICODE_BODY: char = '┃';
    const UNICODE_HALF_BODY_BOTTOM: char = '╻';
    const UNICODE_HALF_BODY_TOP: char = '╹';
    const UNICODE_WICK: char = '│';
    const UNICODE_TOP: char = '╽';
    const UNICODE_BOTTOM: char = '╿';
    const UNICODE_UPPER_WICK: char = '╷';
    const UNICODE_LOWER_WICK: char = '╵';

    pub const MARGIN_TOP: i64 = 3;
    pub const BAR_HEIGHT: i64 = 2;

    pub fn new() -> Self {
        #[cfg(target_os = "windows")]
        control::set_virtual_terminal(true).unwrap();

        Self {
            bullish_color: (52, 208, 88),
            bearish_color: (234, 74, 90),
        }
    }

    fn colorize(&self, candle_type: &CandleType, string: &str) -> String {
        let (ar, ag, ab) = self.bearish_color;
        let (br, bg, bb) = self.bullish_color;

        match candle_type {
            CandleType::Bearish => format!("{}", string.truecolor(ar, ag, ab)),
            CandleType::Bullish => format!("{}", string.truecolor(br, bg, bb)),
        }
    }

    pub fn render(&self, chart: &Chart) {
        let mut output_str = String::new();

        let height =
            chart.terminal_size.1 as i64 - ChartRender::MARGIN_TOP - ChartRender::BAR_HEIGHT;

        for y in (1..height as u16).rev() {
            output_str += "\n";

            output_str += &*YAxis::render_line(y, chart);

            for candle in chart.visible_candles.iter() {
                output_str += &*self.render_candle(&chart.candle_static, candle, y.into(), height);
            }
        }

        output_str += &*self.render_info_bar(chart);

        println!("{}", output_str);
    }

    fn render_candle(
        &self,
        statics: &CandleStatistics,
        candle: &Candle,
        y: i32,
        height: i64,
    ) -> String {
        let height_unit = y as f64;
        let high_y =
            YAxis::price_to_height(candle.high, statics.min_price, statics.max_price, height);
        let low_y =
            YAxis::price_to_height(candle.low, statics.min_price, statics.max_price, height);
        let open_y = YAxis::price_to_height(
            candle.open.max(candle.close),
            statics.min_price,
            statics.max_price,
            height,
        );
        let close_y = YAxis::price_to_height(
            candle.close.min(candle.open),
            statics.min_price,
            statics.max_price,
            height,
        );

        let mut output = ChartRender::UNICODE_VOID;

        // 最高价比当前高度高 并且 高度大于等于开盘价
        if high_y.ceil() >= height_unit && height_unit >= open_y.floor() {
            if open_y - height_unit > 0.75 {
                output = ChartRender::UNICODE_BODY
            } else if open_y - high_y > 0.25 {
                if high_y - height_unit > 0.75 {
                    output = ChartRender::UNICODE_TOP;
                } else {
                    output = ChartRender::UNICODE_HALF_BODY_BOTTOM;
                }
            } else if high_y - height_unit > 0.75 {
                output = ChartRender::UNICODE_WICK;
            } else if high_y - height_unit > 0.25 {
                output = ChartRender::UNICODE_UPPER_WICK;
            }
        } else if open_y.floor() >= height_unit && height_unit >= close_y.ceil() {
            //开盘价大于等于高度 并且 高度大于等于收盘价
            output = ChartRender::UNICODE_BODY;
        } else if close_y.ceil() >= height_unit && height_unit >= low_y.floor() {
            //收盘价大于高度 并且 高度大于最低价
            if close_y - height_unit < 0.25 {
                output = ChartRender::UNICODE_BODY
            } else if close_y - height_unit < 0.75 {
                if close_y - height_unit < 0.25 {
                    output = ChartRender::UNICODE_BOTTOM;
                } else {
                    output = ChartRender::UNICODE_HALF_BODY_TOP;
                }
            } else if low_y - height_unit < 0.25 {
                output = ChartRender::UNICODE_WICK;
            } else if low_y - height_unit < 0.75 {
                output = ChartRender::UNICODE_LOWER_WICK;
            }
        }

        self.colorize(&candle.get_type(), &output.to_string())
    }

    fn render_info_bar(&self, chart: &Chart) -> String {
        let mut output_str = String::new();

        output_str += "\n";
        output_str += &"-".repeat(chart.visible_candles.len() + YAxis::WIDTH as usize);
        output_str += "\n";

        let mut avg_format = format!("{:.2}", chart.candle_static.average);
        avg_format = match chart.candle_static.last_price {
            lp if lp > chart.candle_static.average => avg_format.bold().red(),
            lp if lp < chart.candle_static.average => avg_format.bold().green(),
            _ => avg_format.bold().yellow(),
        }
        .to_string();

        let variation_output = if chart.candle_static.variation > 0.0 {
            ("↖", "green")
        } else {
            ("↙", "red")
        };

        output_str += &format!(
            "{:>width$} | Price: {price} | Highest: {high} | Lowest: {low} | Var.: {var} | Avg.: {avg}",
            "test",
            width = YAxis::MARGIN_RIGHT as usize + 3,
            price = format!("{:.2}", chart.candle_static.last_price).green().bold(),
            high = format!("{:.2}", chart.candle_static.max_price).green().bold(),
            low = format!("{:.2}", chart.candle_static.min_price).red().bold(),
            var = format!("{} {:>+.2}%", variation_output.0, chart.candle_static.variation).color(variation_output.1).bold(),
            avg = avg_format,
        );

        output_str
    }
}
