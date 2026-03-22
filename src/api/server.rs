use crate::core::state::GlobalState;
use crate::monitoring::{HealthChecker, MetricsCollector};
use crate::persistence::Repository;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub state: Arc<Mutex<GlobalState>>,
    pub health_checker: Arc<HealthChecker>,
    pub metrics_collector: Arc<MetricsCollector>,
    pub repository: Repository,
}

pub async fn state_streamer(_state: Arc<Mutex<GlobalState>>) {
    //     println!("[INFO] "State streamer not implemented");
    // TODO: Implement WebSocket streaming for frontend
}

pub async fn start_server(
    state: Arc<Mutex<GlobalState>>,
    health_checker: HealthChecker,
    metrics_collector: MetricsCollector,
    repository: Repository,
) -> Result<(), Box<dyn std::error::Error>> {
    let health_checker_arc = Arc::new(health_checker);
    let metrics_collector_arc = Arc::new(metrics_collector);

    let app_state = AppState {
        state,
        health_checker: Arc::clone(&health_checker_arc),
        metrics_collector: Arc::clone(&metrics_collector_arc),
        repository,
    };

    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(get_health_status))
        .route("/metrics", get(get_metrics))
        .route("/status", get(get_system_status))
        .route("/positions", get(get_positions))
        .route("/orders", get(get_active_orders))
        .route("/performance", get(get_performance_metrics))
        .route("/config", get(get_config).post(update_config))
        .route("/control/start", post(start_trading))
        .route("/control/stop", post(stop_trading))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    //     println!("[INFO] "Starting API server on 0.0.0.0:8000");

    // Start background health monitoring
    tokio::spawn(async move {
        health_checker_arc.start_background_monitoring().await;
    });

    // Start background metrics collection
    tokio::spawn(async move {
        metrics_collector_arc.start_background_collection();
    });

    axum::serve(listener, app).await?;
    Ok(())
}

// Basic health check endpoint
async fn health_check() -> &'static str {
    "CVD Trader Rust API - Healthy"
}

// Comprehensive health status
async fn get_health_status(State(app_state): State<AppState>) -> impl IntoResponse {
    match app_state.health_checker.check_health().await {
        Ok(health) => {
            let status_code = match health.overall {
                crate::monitoring::health::SystemHealth::Healthy => StatusCode::OK,
                crate::monitoring::health::SystemHealth::Degraded => StatusCode::OK,
                crate::monitoring::health::SystemHealth::Unhealthy => {
                    StatusCode::SERVICE_UNAVAILABLE
                }
            };

            (
                status_code,
                Json(json!({
                    "health": health,
                    "status_code": status_code.as_u16(),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })),
            )
        }
        Err(e) => {
            // //             eprintln!("[ERROR]",  "Health check failed", error = %e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Health check failed",
                    "message": e.to_string(),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })),
            )
        }
    }
}

// Prometheus metrics endpoint
async fn get_metrics() -> impl IntoResponse {
    // Return Prometheus-formatted metrics
    // In a real implementation, this would format metrics from the collector
    "# CVD Trader Metrics\n# TODO: Implement Prometheus formatting\n"
}

// System status overview
async fn get_system_status(State(app_state): State<AppState>) -> impl IntoResponse {
    let state = app_state.state.lock().await;
    let db_stats = match app_state.repository.db.get_stats().await {
        Ok(stats) => Some(stats),
        Err(_e) => {
            // //             eprintln!("[ERROR]",  "Failed to get DB stats", error = %e);
            None
        }
    };

    let status = json!({
        "is_running": state.is_running,
        "positions_count": state.positions.len(),
        "active_orders_count": state.active_orders.len(),
        "closed_trades_count": state.closed_trades.len(),
        "wallet_balance": state.wallet_balance,
        "main_wallet_balance": state.main_wallet_balance,
        "config": state.config,
        "database": db_stats.map(|stats| json!({
            "positions": stats.position_count,
            "orders": stats.active_order_count,
            "trades": stats.closed_trade_count,
            "metrics": stats.metric_count,
            "unhealthy_events": stats.unhealthy_events
        })),
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    Json(status)
}

// Get current positions
async fn get_positions(State(app_state): State<AppState>) -> impl IntoResponse {
    let state = app_state.state.lock().await;

    let positions: Vec<_> = state.positions.values().cloned().collect();

    Json(json!({
        "positions": positions,
        "count": positions.len(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// Get active orders
async fn get_active_orders(State(app_state): State<AppState>) -> impl IntoResponse {
    let state = app_state.state.lock().await;

    let orders: Vec<_> = state.active_orders.values().cloned().collect();

    Json(json!({
        "orders": orders,
        "count": orders.len(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// Get performance metrics
async fn get_performance_metrics(State(app_state): State<AppState>) -> impl IntoResponse {
    match app_state.repository.get_recent_metrics(100).await {
        Ok(metrics) => (
            StatusCode::OK,
            Json(json!({
                "metrics": metrics,
                "count": metrics.len(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            })),
        ),
        Err(e) => {
            // //             eprintln!("[ERROR]",  "Failed to get performance metrics", error = %e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to retrieve metrics",
                    "message": e.to_string()
                })),
            )
        }
    }
}

// Get current configuration
async fn get_config(State(app_state): State<AppState>) -> impl IntoResponse {
    match app_state.repository.load_config().await {
        Ok(config) => (
            StatusCode::OK,
            Json(json!({
                "config": config,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })),
        ),
        Err(e) => {
            // //             eprintln!("[ERROR]",  "Failed to load config", error = %e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to load configuration",
                    "message": e.to_string()
                })),
            )
        }
    }
}

// Update configuration
async fn update_config(
    State(app_state): State<AppState>,
    Json(new_values): Json<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let mut state = app_state.state.lock().await;

    // Update in-memory state based on keys
    if let Some(lookback) = new_values.get("lookback").and_then(|v| v.parse().ok()) {
        state.config.strategy.lookback = lookback;
    }
    if let Some(mode) = new_values.get("execution_mode") {
        state.config.execution.mode = mode.clone();
    }

    // Persist to database (using the raw new_values for simplicity in this demo)
    match app_state.repository.save_config(&new_values).await {
        Ok(_) => {
            (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "message": "Configuration updated",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })),
            )
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to save configuration",
                    "message": e.to_string()
                })),
            )
        }
    }
}

// Start trading
async fn start_trading(State(app_state): State<AppState>) -> impl IntoResponse {
    let mut state = app_state.state.lock().await;

    state.start_bot().await;

    //     println!("[INFO] "Trading started via API");
    Json(json!({
        "status": "success",
        "message": "Trading started",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// Stop trading
async fn stop_trading(State(app_state): State<AppState>) -> impl IntoResponse {
    let mut state = app_state.state.lock().await;

    state.stop_bot().await;

    //     println!("[INFO] "Trading stopped via API");
    Json(json!({
        "status": "success",
        "message": "Trading stopped",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
