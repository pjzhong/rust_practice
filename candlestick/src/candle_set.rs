use std::cmp::Ordering;

#[derive(Debug, Copy, Clone)]
pub struct Candle {
    /// 开盘价
    pub open: f64,
    /// 最高价
    pub high: f64,
    /// 最低价
    pub low: f64,
    /// 收盘价
    pub close: f64,
    /// 时间戳
    pub timestamp: Option<i64>,
}

pub(crate) enum CandleType {
    /// 熊市
    Bearish,
    /// 牛市
    Bullish,
}

impl Candle {
    pub fn new(open: f64, high: f64, low: f64, close: f64) -> Candle {
        Candle {
            open,
            high,
            low,
            close,
            timestamp: None,
        }
    }

    pub(crate) fn get_type(&self) -> CandleType {
        if self.open < self.close {
            CandleType::Bearish
        } else {
            CandleType::Bullish
        }
    }
}

#[derive(Debug)]
pub struct CandleStatistics {
    pub min_price: f64,
    pub max_price: f64,

    pub variation: f64,
    pub average: f64,
    pub last_price: f64,
}

impl CandleStatistics {
    pub fn new() -> CandleStatistics {
        Self {
            min_price: 0.0,
            max_price: 0.0,
            variation: 0.0,
            average: 0.0,
            last_price: 0.0,
        }
    }

    pub fn compute_candles(&mut self, candles: &[Candle]) {
        self.compute_last_price(candles);
        self.compute_variation(candles);
        self.compute_average(candles);
        self.compute_min_and_max(candles);
    }

    fn compute_last_price(&mut self, candles: &[Candle]) {
        if let Some(candle) = candles.last() {
            self.last_price = candle.close;
        } else {
            self.last_price = 0.0;
        }
    }

    fn compute_variation(&mut self, candles: &[Candle]) {
        let open = candles.first();
        let close = candles.last();

        if open.is_none() || close.is_none() {
            self.variation = 0.0;
            return;
        }

        let open = open.unwrap();
        let close = close.unwrap();

        self.variation = ((close.close - open.open) / open.open) * 100.0;
    }

    fn compute_average(&mut self, candles: &[Candle]) {
        if candles.is_empty() {
            self.average = 0.0;
        } else {
            let sum = candles.iter().fold(0.0, |acc, c| acc + c.close);
            self.average = sum / candles.len() as f64;
        }
    }

    fn compute_min_and_max(&mut self, candles: &[Candle]) {
        self.max_price = candles
            .iter()
            .max_by(|a, b| a.high.partial_cmp(&b.high).unwrap_or(Ordering::Equal))
            .map(|c| c.high)
            .unwrap_or(0.0);

        self.min_price = candles
            .iter()
            .min_by(|a, b| a.high.partial_cmp(&b.high).unwrap_or(Ordering::Equal))
            .map(|c| c.low)
            .unwrap_or(0.0);
    }
}
