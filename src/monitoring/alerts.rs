use crate::monitoring::health::{ComponentStatus, HealthStatus, SystemHealth};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Alert {
    pub id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub component: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub enum AlertType {
    HealthCheckFailed,
    PerformanceDegraded,
    SystemUnhealthy,
    DatabaseError,
    NetworkError,
    HighLatency,
    MemoryPressure,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct AlertManager {
    active_alerts: HashMap<String, Alert>,
    alert_handlers: Vec<Box<dyn AlertHandler + Send + Sync>>,
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            active_alerts: HashMap::new(),
            alert_handlers: Vec::new(),
        }
    }

    pub fn register_handler(&mut self, handler: Box<dyn AlertHandler + Send + Sync>) {
        self.alert_handlers.push(handler);
    }

    pub async fn process_health_status(&mut self, status: &HealthStatus) {
        // Check overall system health
        match status.overall {
            SystemHealth::Unhealthy => {
                self.raise_alert(
                    "system_unhealthy".to_string(),
                    AlertType::SystemUnhealthy,
                    AlertSeverity::Critical,
                    "system".to_string(),
                    format!(
                        "System is unhealthy: {} components affected",
                        status
                            .components
                            .iter()
                            .filter(|c| matches!(c.status, ComponentStatus::Unhealthy))
                            .count()
                    ),
                )
                .await;
            }
            SystemHealth::Degraded => {
                self.raise_alert(
                    "system_degraded".to_string(),
                    AlertType::SystemUnhealthy,
                    AlertSeverity::High,
                    "system".to_string(),
                    format!(
                        "System is degraded: {} components affected",
                        status
                            .components
                            .iter()
                            .filter(|c| matches!(c.status, ComponentStatus::Degraded))
                            .count()
                    ),
                )
                .await;
            }
            SystemHealth::Healthy => {
                // System is healthy, resolve any system alerts
                self.resolve_alert("system_unhealthy").await;
                self.resolve_alert("system_degraded").await;
            }
        }

        // Check individual components
        for component in &status.components {
            let alert_key = format!("{}_{}", component.name, "health");

            match component.status {
                ComponentStatus::Unhealthy => {
                    self.raise_alert(
                        alert_key,
                        AlertType::HealthCheckFailed,
                        AlertSeverity::High,
                        component.name.clone(),
                        component
                            .message
                            .clone()
                            .unwrap_or_else(|| "Component is unhealthy".to_string()),
                    )
                    .await;
                }
                ComponentStatus::Degraded => {
                    self.raise_alert(
                        alert_key,
                        AlertType::PerformanceDegraded,
                        AlertSeverity::Medium,
                        component.name.clone(),
                        component
                            .message
                            .clone()
                            .unwrap_or_else(|| "Component is degraded".to_string()),
                    )
                    .await;
                }
                ComponentStatus::Healthy => {
                    // Component is healthy, resolve any component alerts
                    self.resolve_alert(&alert_key).await;
                }
            }
        }
    }

    pub async fn raise_alert(
        &mut self,
        id: String,
        alert_type: AlertType,
        severity: AlertSeverity,
        component: String,
        message: String,
    ) {
        // Check if alert is already active
        if self.active_alerts.contains_key(&id) {
            return; // Alert already active
        }

        let alert = Alert {
            id: id.clone(),
            alert_type,
            severity: severity.clone(),
            component,
            message,
            timestamp: Utc::now(),
            resolved: false,
            resolved_at: None,
        };

        self.active_alerts.insert(id.clone(), alert.clone());

        // Log the alert
        let severity_str = match severity {
            AlertSeverity::Low => "LOW",
            AlertSeverity::Medium => "MEDIUM",
            AlertSeverity::High => "HIGH",
            AlertSeverity::Critical => "CRITICAL",
        };

        // eprintln!("[ERROR] Alert raised: {} {} {} {}", id, severity_str, alert.component, alert.message);

        // Notify all handlers
        for handler in &self.alert_handlers {
            if let Err(e) = handler.handle_alert(&alert).await {
                // eprintln!("[ERROR] Alert handler failed: {} {}", handler.name(), e);
            }
        }
    }

    pub async fn resolve_alert(&mut self, id: &str) {
        if let Some(alert) = self.active_alerts.get_mut(id) {
            alert.resolved = true;
            alert.resolved_at = Some(Utc::now());

            // println!("[INFO] Alert resolved: {} {} {}", id, alert.component, (Utc::now() - alert.timestamp).num_minutes());

            // Notify handlers of resolution
            for handler in &self.alert_handlers {
                if let Err(e) = handler.handle_resolution(alert).await {
                    // eprintln!("[ERROR] Alert resolution handler failed: {} {}", handler.name(), e);
                }
            }

            // Remove from active alerts
            self.active_alerts.remove(id);
        }
    }

    pub fn get_active_alerts(&self) -> Vec<&Alert> {
        self.active_alerts.values().collect()
    }

    pub fn get_alerts_by_component(&self, component: &str) -> Vec<&Alert> {
        self.active_alerts
            .values()
            .filter(|alert| alert.component == component)
            .collect()
    }

    pub fn get_alerts_by_severity(&self, severity: AlertSeverity) -> Vec<&Alert> {
        self.active_alerts
            .values()
            .filter(|alert| alert.severity == severity)
            .collect()
    }
}

#[async_trait::async_trait]
pub trait AlertHandler: Send + Sync {
    fn name(&self) -> &'static str;
    async fn handle_alert(&self, alert: &Alert) -> Result<(), Box<dyn std::error::Error>>;
    async fn handle_resolution(&self, alert: &Alert) -> Result<(), Box<dyn std::error::Error>>;
}

// Console alert handler (logs to console)
pub struct ConsoleAlertHandler;

impl ConsoleAlertHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl AlertHandler for ConsoleAlertHandler {
    fn name(&self) -> &'static str {
        "console"
    }

    async fn handle_alert(&self, alert: &Alert) -> Result<(), Box<dyn std::error::Error>> {
        // println!("🚨 ALERT: [{}] {} - {}: {}", match alert.severity { AlertSeverity::Low => "LOW", AlertSeverity::Medium => "MEDIUM", AlertSeverity::High => "HIGH", AlertSeverity::Critical => "CRITICAL" }, alert.component, alert.id, alert.message);
        Ok(())
    }

    async fn handle_resolution(&self, alert: &Alert) -> Result<(), Box<dyn std::error::Error>> {
        //         println!("✅ RESOLVED: {} - {}", alert.component, alert.id);
        Ok(())
    }
}

// Email alert handler (placeholder for actual email implementation)
pub struct EmailAlertHandler {
    recipients: Vec<String>,
}

impl EmailAlertHandler {
    pub fn new(recipients: Vec<String>) -> Self {
        Self { recipients }
    }
}

#[async_trait::async_trait]
impl AlertHandler for EmailAlertHandler {
    fn name(&self) -> &'static str {
        "email"
    }

    async fn handle_alert(&self, alert: &Alert) -> Result<(), Box<dyn std::error::Error>> {
        // In a real implementation, this would send emails
        // For now, just log the intent
        // println!("[INFO] Email alert would be sent: {:?} {} {:?} {}", self.recipients, alert.id, alert.severity, alert.message);
        Ok(())
    }

    async fn handle_resolution(&self, alert: &Alert) -> Result<(), Box<dyn std::error::Error>> {
        // println!("[INFO] Email resolution would be sent: {:?} {}", self.recipients, alert.id);
        Ok(())
    }
}

// Webhook alert handler (placeholder for HTTP webhook notifications)
pub struct WebhookAlertHandler {
    webhook_url: String,
}

impl WebhookAlertHandler {
    pub fn new(webhook_url: String) -> Self {
        Self { webhook_url }
    }
}

#[async_trait::async_trait]
impl AlertHandler for WebhookAlertHandler {
    fn name(&self) -> &'static str {
        "webhook"
    }

    async fn handle_alert(&self, alert: &Alert) -> Result<(), Box<dyn std::error::Error>> {
        // In a real implementation, this would POST to the webhook URL
        // println!("[INFO] Webhook alert would be sent: {} {} {:?} {}", self.webhook_url, alert.id, alert.severity, alert.message);
        Ok(())
    }

    async fn handle_resolution(&self, alert: &Alert) -> Result<(), Box<dyn std::error::Error>> {
        // println!("[INFO] Webhook resolution would be sent: {} {}", self.webhook_url, alert.id);
        Ok(())
    }
}
