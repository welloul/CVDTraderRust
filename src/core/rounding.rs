use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct AssetInfo {
    pub sz_decimals: i32,
    pub px_decimals: i32,
    pub tick_size: f64,
}

#[derive(Debug)]
pub struct RoundingUtil {
    asset_info: HashMap<String, AssetInfo>,
}

impl RoundingUtil {
    pub fn new(meta_info: Option<Value>) -> Self {
        let mut util = RoundingUtil {
            asset_info: HashMap::new(),
        };
        util.parse_meta(meta_info);
        util
    }

    fn parse_meta(&mut self, meta_info: Option<Value>) {
        if meta_info.is_none() {
            // Use default values for common coins when meta is unavailable
            self.asset_info.insert("BTC".to_string(), AssetInfo {
                sz_decimals: 2,
                px_decimals: 2,
                tick_size: 0.01,
            });
            self.asset_info.insert("ETH".to_string(), AssetInfo {
                sz_decimals: 4,
                px_decimals: 2,
                tick_size: 0.01,
            });
            self.asset_info.insert("SOL".to_string(), AssetInfo {
                sz_decimals: 2,
                px_decimals: 2,
                tick_size: 0.01,
            });
            return;
        }

        let meta = meta_info.unwrap();
        if let Some(universe) = meta.get("universe").and_then(|u| u.as_array()) {
            for asset in universe {
                if let Some(name) = asset.get("name").and_then(|n| n.as_str()) {
                    let sz_decimals = asset.get("szDecimals")
                        .and_then(|s| s.as_i64())
                        .unwrap_or(2) as i32;
                    let tick_size = asset.get("tickSize")
                        .and_then(|t| t.as_f64())
                        .unwrap_or(0.01);

                    let px_decimals = if 5 - sz_decimals > 0 { 5 - sz_decimals } else { 2 };

                    self.asset_info.insert(name.to_string(), AssetInfo {
                        sz_decimals,
                        px_decimals,
                        tick_size,
                    });
                }
            }
        }
    }

    pub fn round_size(&self, coin: &str, sz: f64) -> String {
        let info = match self.asset_info.get(coin) {
            Some(info) => info,
            None => return sz.to_string(), // Fallback
        };

        let decimals = info.sz_decimals;
        let factor = 10f64.powi(decimals);
        let rounded = (sz * factor).floor() / factor;
        format!("{:.1$}", rounded, decimals as usize)
    }

    pub fn round_price(&self, coin: &str, px: f64) -> String {
        let info = match self.asset_info.get(coin) {
            Some(info) => info,
            None => return px.to_string(), // Fallback
        };

        let tick_size = info.tick_size;
        let rounded = if tick_size > 0.0 {
            ((px / tick_size).round()) * tick_size
        } else {
            px
        };

        if tick_size >= 1.0 {
            (rounded as i64).to_string()
        } else {
            // Count decimal places needed for tick size
            let tick_str = format!("{:.10}", tick_size);
            let decimals = if let Some(dot_pos) = tick_str.find('.') {
                tick_str[dot_pos + 1..].trim_end_matches('0').len()
            } else {
                2
            };
            format!("{:.1$}", rounded, decimals)
        }
    }

    pub fn format_for_api(&self, num: f64) -> String {
        let s = num.to_string();
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            s
        }
    }
}