use tracing::{info, warn, error};
use cvd_trader_rust::{
    api::server,
    core::{config::Config, logger::init_logger, rounding::RoundingUtil, state::GlobalState},
    execution::{gateway::ExecutionGateway, ttl::OrderTTLTracker},
    hyperliquid::{Exchange, Info, constants::*},
    market_data::handler::MarketDataHandler,
    monitoring::{health::HealthChecker, metrics::MetricsCollector},
    persistence::{database::Database, repository::Repository},
    risk::manager::RiskManager,
    strategy::module::StrategyModule,
};
use dotenvy;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logger();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Load configuration
    let config = Config::load();
    info!("Configuration loaded: {:?}", config);

    // Initialize global state
    let state = Arc::new(Mutex::new(GlobalState::new()));

    // Initialize exchange clients
    let (exchange, wallet_address, info) = if config.execution.mode != "dryrun" {
        let secret = std::env::var("HYPERLIQUID_SECRET_KEY").expect("HYPERLIQUID_SECRET_KEY required");
        let address = std::env::var("HYPERLIQUID_WALLET_ADDRESS").expect("HYPERLIQUID_WALLET_ADDRESS required");
        
        let account = cvd_trader_rust::hyperliquid::client::Account::from_key(&secret);
        let exch = Exchange::new(account, MAINNET_API_URL);
        let info_client = Info::new(MAINNET_API_URL).await.expect("Failed to init Info client");
        
        (Some(exch), Some(address), Some(info_client))
    } else {
        (None, None, None)
    };

    // Fetch metadata for rounding
    let meta_info = if let Some(ref client) = exchange {
        // TODO: Implement get_meta_info on HyperliquidClient
        None
    } else {
        None
    };
    let rounding_util = RoundingUtil::new(meta_info);

    // Sync initial state
    if let (Some(ref info), Some(ref addr)) = (&info, &wallet_address) {
        let mut state_lock = state.lock().await;
        state_lock.sync_state(info, addr).await;
    }

    // Initialize database
    let db_path = "cvd_trader.db".to_string();
    let database = Database::new(db_path.clone());
    database.initialize()?;
    let repository = Arc::new(Repository::new(database));

    // Initialize monitoring
    let health_checker = HealthChecker::new(Arc::clone(&state), &repository);
    let metrics_collector = Arc::new(MetricsCollector::new((*repository).clone()));

    // 7. Initialize Strategy & Execution
    let risk_manager = Arc::new(Mutex::new(RiskManager::new(&config.risk)));

    // Initialize TTL tracker
    let ttl_tracker = if exchange.is_some() {
        let tracker = Arc::new(Mutex::new(OrderTTLTracker::new(Arc::clone(&state), None)));
        // Start TTL tracker task
        let tracker_clone = Arc::clone(&tracker);
        tokio::spawn(async move {
            let mut tracker = tracker_clone.lock().await;
            tracker.start().await;
        });
        Some(tracker)
    } else {
        None
    };

    let gateway = if let Some(exch) = exchange {
        Some(Arc::new(Mutex::new(ExecutionGateway::new(
            exch,
            rounding_util,
            Arc::clone(&state),
            ttl_tracker.as_ref().map(Arc::clone),
        ))))
    } else {
        warn!("Execution gateway is disabled (read-only mode).");
        None
    };

    // Set gateway on TTL tracker
    if let (Some(ref tracker), Some(ref gateway)) = (&ttl_tracker, &gateway) {
        let mut tracker = tracker.lock().await;
        tracker.gateway = Some(Arc::clone(gateway));
    }

    let strategy = Arc::new(Mutex::new(StrategyModule::new(
        Arc::clone(&state),
        gateway.as_ref().map(Arc::clone),
        Arc::clone(&risk_manager),
        ttl_tracker.as_ref().map(Arc::clone),
    )));

    // 8. Initialize Market Data Handlers for all configured coins
    let target_coins = config.general.target_coins.clone();

    let mut handlers = Vec::new();
    let mut md_tasks = Vec::new();

    for coin in &target_coins {
        let coin_name = coin.clone();
        info!("Initializing MarketDataHandler for {}", &coin_name);
        let mut md_handler = MarketDataHandler::new(coin_name.clone(), Arc::clone(&state));
        let strategy_clone = Arc::clone(&strategy);
        let metrics_clone = Arc::clone(&metrics_collector);
        let coin_callback = coin_name.clone();

        md_handler.add_callback(move |event_value| {
            let strategy_clone = Arc::clone(&strategy_clone);
            let metrics_clone = Arc::clone(&metrics_clone);
            let coin_inner = coin_callback.clone();

            async move {
                if let Some(event) = cvd_trader_rust::market_data::event::MarketDataEvent::from_value(event_value) {
                    // Record latency
                    metrics_clone.record_market_data_latency(&coin_inner, event.latency_ms);

                    let mut strategy = strategy_clone.lock().await;
                    strategy.on_market_data(event).await;
                }
            }
        });

        let handler = Arc::new(Mutex::new(md_handler));
        handlers.push(Arc::clone(&handler));

        let handler_clone = Arc::clone(&handler);
        let task = tokio::spawn(async move {
            let mut handler = handler_clone.lock().await;
            handler.connect().await;
        });
        md_tasks.push(task);
    }

    // 9. Start API Server with monitoring
    info!("Starting API server...");
    let state_server = Arc::clone(&state);
    let repo_server = (*repository).clone();
    let metrics_server = (*metrics_collector).clone();
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server::start_server(
            state_server,
            health_checker,
            metrics_server,
            repo_server,
        )
        .await
        {
            error!("API server error: {}", e);
        }
    });

    // 10. Start background tasks
    info!("Starting background tasks...");

    // Start alert manager background monitoring
    let alert_manager_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            // Alert manager runs in the health checker background task
        }
    });

    // Start periodic state sync
    let state_sync = Arc::clone(&state);
    let wallet_addr = wallet_address.clone();
    let info_sync = info.clone();
    let sync_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30)); // More frequent sync
        loop {
            interval.tick().await;
            if let (Some(ref addr), Some(ref info)) = (&wallet_addr, &info_sync) {
                let mut state = state_sync.lock().await;
                state.sync_state(info, addr).await;
                info!("Periodic state sync completed");
            }
        }
    });

    // Start database cleanup task
    let cleanup_repo = Arc::clone(&repository);
    let cleanup_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600)); // Hourly cleanup
        loop {
            interval.tick().await;
            if let Err(e) = cleanup_repo.cleanup_old_data(30).await {
                warn!("Database cleanup failed: {}", e);
            } else {
                info!("Database cleanup completed");
            }
        }
    });

    info!("CVD Trader Rust fully initialized and running");
    info!("API server available at http://0.0.0.0:8000");
    info!("Health checks at http://0.0.0.0:8000/health");
    info!("Metrics at http://0.0.0.0:8000/metrics");

    // Wait for all market data tasks
    for task in md_tasks {
        let _ = task.await;
    }

    // Wait for background tasks (they run indefinitely)
    let _ = tokio::try_join!(
        server_handle,
        alert_manager_handle,
        sync_handle,
        cleanup_handle
    );

    Ok(())
}
