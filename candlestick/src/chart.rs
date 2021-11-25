use terminal_size::{terminal_size, Height, Width};

use crate::candle_set::{Candle, CandleStatistics};
use crate::chart_render::ChartRender;
use crate::yaxis::YAxis;

#[derive(Debug)]
pub struct Chart {
    pub(crate) candles: Vec<Candle>,
    pub(crate) visible_candles: Vec<Candle>,
    pub(crate) candle_static: CandleStatistics,
    ///(width, height)
    pub(crate) terminal_size: (i64, i64),

    chart_render: ChartRender,
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

        let visible_candles = Chart::compute_visible_candles(w.0 as i64, &candles);

        Self {
            candles,
            visible_candles,
            candle_static,
            terminal_size: (w.0 as i64, h.0 as i64),
            chart_render: ChartRender::new(),
        }
    }

    fn compute_visible_candles(terminal_width: i64, candles: &Vec<Candle>) -> Vec<Candle> {
        let nb_candles = candles.len();
        let nb_visible_candles = terminal_width - YAxis::WIDTH;

        candles
            .iter()
            .skip((nb_candles as i64 - nb_visible_candles as i64).max(0) as usize)
            .cloned()
            .collect::<Vec<Candle>>()
    }

    pub fn draw(&self) {
        self.chart_render.render(self);
    }
}
