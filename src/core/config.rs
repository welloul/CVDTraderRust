use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub strategy: StrategyConfig,
    pub risk: RiskConfig,
    pub execution: ExecutionConfig,
    pub general: GeneralConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StrategyConfig {
    pub lookback: usize,
    pub cvd_exhaustion_ratio: f64,
    pub cvd_absorption_pctile: f64,
    pub fixed_fee_rate: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RiskConfig {
    pub max_allowed_latency_ms: f64,
    pub consecutive_failures_threshold: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutionConfig {
    pub mode: String,
    pub default_slippage_pct: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeneralConfig {
    pub max_latency_ms: f64,
    pub target_coins: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            strategy: StrategyConfig::default(),
            risk: RiskConfig::default(),
            execution: ExecutionConfig::default(),
            general: GeneralConfig::default(),
        }
    }
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            lookback: 20,
            cvd_exhaustion_ratio: 0.70,
            cvd_absorption_pctile: 0.90,
            fixed_fee_rate: 0.0003,
        }
    }
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_allowed_latency_ms: 1000.0,
            consecutive_failures_threshold: 3,
        }
    }
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            mode: "dryrun".to_string(),
            default_slippage_pct: 0.001,
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            max_latency_ms: 5000.0,
            target_coins: vec!["SOL".to_string(), "ZEC".to_string(), "HYPE".to_string(), "XMR".to_string(), "LINK".to_string(), "XLM".to_string(), "AVAX".to_string(), "TON".to_string(), "TAO".to_string()],
        }
    }
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn load() -> Self {
        // Try to load from config.toml, fallback to defaults
        Self::from_file("config.toml").unwrap_or_default()
    }
}