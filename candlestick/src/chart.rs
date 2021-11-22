use terminal_size::{terminal_size, Height, Width};

use crate::candle_set::{Candle, CandleStatistics};

#[derive(Debug)]
pub struct Chart {
    pub candles: Vec<Candle>,
    pub candle_static: CandleStatistics,
    ///(width, height)
    pub terminal_size: (i64, i64),
}

impl Chart {
    pub fn new(candles: Vec<Candle>) -> Self {
        let (w, h) = if let Some(size) = terminal_size() {
            size
        } else {
            (Width(100), Height(20))
        };

        let mut candle_static = CandleStatistics::new();
        candle_static.compute_candles(&candles);

        Self {
            candles,
            candle_static,
            terminal_size: (w.0 as i64, h.0 as i64),
        }
    }
}
