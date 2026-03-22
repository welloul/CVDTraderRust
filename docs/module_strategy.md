# Module: Strategy

## Responsibility
The Strategy module implements core trading logic, transforming `MarketDataEvent` flows into actionable buy/sell signals. It manages asset-specific `Candle` history and evaluates signal conditions like price/CVD divergence.

## Key Logic & Functions

### StrategyModule
The stateful engine for trade decisions.
- **`on_market_data(event: MarketDataEvent)`**: The main entry point for strategy evaluation.
- **`evaluate_signal(...)`**: Core logic for opening positions based on OHLCV + CVD dynamics.
- **`check_sl_tp(...)`**: Evaluates active positions against risk parameters (Stop Loss/Take Profit).

### Signal Detection
- **`detect_swing_high/low`**: Identifies 5-bar pivot points.
- **`is_cvd_exhaustion`**: Compares Price Highs to CVD Highs to detect exhaustion/absorption.

## Testing: Scenario-based Framework
The module contains a `#[cfg(test)]` suite that allows the injection of syntheticOHLCV + CVD data.
- `test_swing_high_cvd_exhaustion`: Mocks a higher price high with a lower CVD high to confirm a short signal.
- `test_swing_low_cvd_exhaustion`: Mocks a lower price low with a higher CVD low to confirm a long signal.

## Hurdles
* **Race Conditions**: `on_market_data` requires asynchronous access to the shared `GlobalState`.
* **Config Sync**: Strategy parameters were previously static; the module now reads from `GlobalState.config` for runtime observability.

## Future Roadmap
- [ ] Implement **VWAP-relative** position sizing for improved risk control.
- [ ] Add **Trailing Stop Loss** logic to capture momentum.
- [ ] Integrate with **Hyperliquid L2 Book** for liquidity-aware signal confirmation.