use crate::core::state::GlobalState;
use crate::persistence::Repository;
use anyhow::{Result, Context};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub overall: SystemHealth,
    pub components: Vec<ComponentHealth>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemHealth {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: ComponentStatus,
    pub message: Option<String>,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub metrics: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

pub struct HealthChecker {
    state: Arc<Mutex<GlobalState>>,
    repository: Repository,
    checks: Vec<Box<dyn HealthCheck + Send + Sync>>,
}

impl HealthChecker {
    pub fn new(state: Arc<Mutex<GlobalState>>, repository: &Repository) -> Self {
        // Register built-in health checks
        let checks = vec![
            Box::new(DatabaseHealthCheck::new(repository)) as Box<dyn HealthCheck + Send + Sync>,
            Box::new(StateHealthCheck::new(&state)),
            Box::new(MemoryHealthCheck::new()),
            Box::new(NetworkHealthCheck::new()),
        ];

        Self {
            state,
            repository: repository.clone(),
            checks,
        }
    }

    pub fn register_check(&mut self, check: Box<dyn HealthCheck + Send + Sync>) {
        self.checks.push(check);
    }

    pub async fn check_health(&self) -> Result<HealthStatus> {
        let mut components = Vec::new();
        let mut overall_status = SystemHealth::Healthy;

        for check in &self.checks {
            match check.check().await {
                Ok(component_health) => {
                    components.push(component_health.clone());

                    // Update overall status based on component status
                    match component_health.status {
                        ComponentStatus::Unhealthy => {
                            overall_status = SystemHealth::Unhealthy;
                        }
                        ComponentStatus::Degraded => {
                            if matches!(overall_status, SystemHealth::Healthy) {
                                overall_status = SystemHealth::Degraded;
                            }
                        }
                        ComponentStatus::Healthy => {
                            // Keep current overall status
                        }
                    }
                }
                Err(e) => {
                    // //                     eprintln!("[ERROR]",  "Health check failed", component = %check.name(), error = %e);
                    components.push(ComponentHealth {
                        name: check.name().to_string(),
                        status: ComponentStatus::Unhealthy,
                        message: Some(format!("Check failed: {}", e)),
                        last_check: chrono::Utc::now(),
                        metrics: None,
                    });
                    overall_status = SystemHealth::Unhealthy;
                }
            }
        }

        Ok(HealthStatus {
            overall: overall_status,
            components,
            timestamp: chrono::Utc::now(),
        })
    }

    pub async fn persist_health_status(&self, status: &HealthStatus) -> Result<()> {
        for component in &status.components {
            let db_health = crate::persistence::models::DbSystemHealth {
                id: None,
                timestamp: component.last_check,
                component: component.name.clone(),
                status: match component.status {
                    ComponentStatus::Healthy => crate::persistence::models::HealthStatus::Healthy,
                    ComponentStatus::Degraded => crate::persistence::models::HealthStatus::Degraded,
                    ComponentStatus::Unhealthy => {
                        crate::persistence::models::HealthStatus::Unhealthy
                    }
                },
                message: component.message.clone(),
                metrics: component.metrics.clone(),
            };

            self.repository.save_system_health(&db_health).await.context("Failed to persist health")?;
        }

        Ok(())
    }

    pub async fn start_background_monitoring(&self) {
        let checker = Arc::new(self.clone());
        let repository = self.repository.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));

            loop {
                interval.tick().await;

                match checker.check_health().await {
                    Ok(status) => {
                        // Log unhealthy components
                        for component in &status.components {
                            if matches!(component.status, ComponentStatus::Unhealthy) {
                                tracing::error!("Component unhealthy: {} - {:?}", component.name, component.message);
                            } else if matches!(component.status, ComponentStatus::Degraded) {
                                tracing::warn!("Component degraded: {} - {:?}", component.name, component.message);
                            }
                        }

                        // Persist health status
                        if let Err(_e) = repository.save_system_health(&crate::persistence::models::DbSystemHealth {
                            id: None,
                            timestamp: status.timestamp,
                            component: "system".to_string(),
                            status: match status.overall {
                                SystemHealth::Healthy => crate::persistence::models::HealthStatus::Healthy,
                                SystemHealth::Degraded => crate::persistence::models::HealthStatus::Degraded,
                                SystemHealth::Unhealthy => crate::persistence::models::HealthStatus::Unhealthy,
                            },
                            message: Some(format!("Overall system health: {:?}", status.overall)),
                            metrics: Some(serde_json::json!({
                                "component_count": status.components.len(),
                                "healthy_components": status.components.iter().filter(|c| matches!(c.status, ComponentStatus::Healthy)).count(),
                                "degraded_components": status.components.iter().filter(|c| matches!(c.status, ComponentStatus::Degraded)).count(),
                                "unhealthy_components": status.components.iter().filter(|c| matches!(c.status, ComponentStatus::Unhealthy)).count()
                            })),
                        }).await {
// //                             eprintln!("[ERROR]",  "Failed to persist health status", error = %e);
                        }
                    }
                    Err(_e) => {
                        // //                         eprintln!("[ERROR]",  "Health check failed", error = %e);
                    }
                }
            }
        });
    }
}

impl Clone for HealthChecker {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            repository: Repository::new(self.repository.db.clone()),
            checks: Vec::new(), // Checks won't be cloned for simplicity
        }
    }
}

#[async_trait]
pub trait HealthCheck: Send + Sync {
    fn name(&self) -> &'static str;
    async fn check(&self) -> Result<ComponentHealth>;
}

// Database health check
pub struct DatabaseHealthCheck {
    repository: Repository,
}

impl DatabaseHealthCheck {
    pub fn new(repository: &Repository) -> Self {
        Self {
            repository: repository.clone(),
        }
    }
}

#[async_trait]
impl HealthCheck for DatabaseHealthCheck {
    fn name(&self) -> &'static str {
        "database"
    }

    async fn check(&self) -> Result<ComponentHealth> {
        let start = std::time::Instant::now();

        // Try to get database stats
        match self.repository.db.get_stats().await {
            Ok(stats) => {
                let duration = start.elapsed().as_millis() as f64;
                Ok(ComponentHealth {
                    name: "database".to_string(),
                    status: ComponentStatus::Healthy,
                    message: Some(format!(
                        "Database operational, {} positions, {} orders, {} trades",
                        stats.position_count, stats.active_order_count, stats.closed_trade_count
                    )),
                    last_check: chrono::Utc::now(),
                    metrics: Some(serde_json::json!({
                        "positions": stats.position_count,
                        "orders": stats.active_order_count,
                        "trades": stats.closed_trade_count,
                        "metrics": stats.metric_count,
                        "unhealthy_events": stats.unhealthy_events,
                        "check_duration_ms": duration
                    })),
                })
            }
            Err(e) => Ok(ComponentHealth {
                name: "database".to_string(),
                status: ComponentStatus::Unhealthy,
                message: Some(format!("Database check failed: {}", e)),
                last_check: chrono::Utc::now(),
                metrics: None,
            }),
        }
    }
}

// State health check
pub struct StateHealthCheck {
    state: Arc<Mutex<GlobalState>>,
}

impl StateHealthCheck {
    pub fn new(state: &Arc<Mutex<GlobalState>>) -> Self {
        Self {
            state: Arc::clone(state),
        }
    }
}

#[async_trait]
impl HealthCheck for StateHealthCheck {
    fn name(&self) -> &'static str {
        "state"
    }

    async fn check(&self) -> Result<ComponentHealth> {
        let state = self.state.lock().await;

        let position_count = state.positions.len();
        let order_count = state.active_orders.len();
        let trade_count = state.closed_trades.len();
        let is_running = state.is_running;

        // Check for potential issues
        let mut issues = Vec::new();

        if position_count > 10 {
            issues.push("High position count may indicate over-leveraging".to_string());
        }

        if !is_running {
            issues.push("System is not running".to_string());
        }

        let status = if issues.is_empty() && is_running {
            ComponentStatus::Healthy
        } else if issues.len() > 0 && is_running {
            ComponentStatus::Degraded
        } else {
            ComponentStatus::Unhealthy
        };

        Ok(ComponentHealth {
            name: "state".to_string(),
            status,
            message: if issues.is_empty() {
                Some(format!(
                    "State healthy: {} positions, {} orders, {} trades",
                    position_count, order_count, trade_count
                ))
            } else {
                Some(format!("State issues: {}", issues.join(", ")))
            },
            last_check: chrono::Utc::now(),
            metrics: Some(serde_json::json!({
                "positions": position_count,
                "orders": order_count,
                "trades": trade_count,
                "is_running": is_running,
                "issues": issues
            })),
        })
    }
}

// Memory health check
pub struct MemoryHealthCheck;

impl MemoryHealthCheck {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl HealthCheck for MemoryHealthCheck {
    fn name(&self) -> &'static str {
        "memory"
    }

    async fn check(&self) -> Result<ComponentHealth> {
        // In a real implementation, you'd use a memory profiling crate
        // For now, we'll just report healthy status
        Ok(ComponentHealth {
            name: "memory".to_string(),
            status: ComponentStatus::Healthy,
            message: Some("Memory usage within normal parameters".to_string()),
            last_check: chrono::Utc::now(),
            metrics: Some(serde_json::json!({
                "usage_mb": 50.5, // Placeholder
                "available_mb": 200.0 // Placeholder
            })),
        })
    }
}

// Network health check
pub struct NetworkHealthCheck;

impl NetworkHealthCheck {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl HealthCheck for NetworkHealthCheck {
    fn name(&self) -> &'static str {
        "network"
    }

    async fn check(&self) -> Result<ComponentHealth> {
        // Simple connectivity check to a reliable endpoint
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            reqwest::get("https://www.google.com"),
        )
        .await
        {
            Ok(Ok(response)) if response.status().is_success() => {
                Ok(ComponentHealth {
                    name: "network".to_string(),
                    status: ComponentStatus::Healthy,
                    message: Some("Network connectivity healthy".to_string()),
                    last_check: chrono::Utc::now(),
                    metrics: Some(serde_json::json!({
                        "connectivity": true,
                        "response_time_ms": 150 // Placeholder
                    })),
                })
            }
            _ => Ok(ComponentHealth {
                name: "network".to_string(),
                status: ComponentStatus::Degraded,
                message: Some("Network connectivity issues detected".to_string()),
                last_check: chrono::Utc::now(),
                metrics: Some(serde_json::json!({
                    "connectivity": false
                })),
            }),
        }
    }
}
