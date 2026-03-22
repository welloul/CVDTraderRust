use reqwest::Client;
use serde_json::Value;
use anyhow::Result;

#[derive(Clone)]
pub struct Account {
    // Placeholder for account implementation
    pub address: String,
    pub secret_key: String,
}

impl Account {
    pub fn from_key(secret_key: &str) -> Self {
        // TODO: Implement proper key handling
        Self {
            address: "placeholder_address".to_string(),
            secret_key: secret_key.to_string(),
        }
    }
}

pub struct Exchange {
    client: Client,
    account: Account,
    base_url: String,
}

impl Exchange {
    pub fn new(account: Account, base_url: &str) -> Self {
        Self {
            client: Client::new(),
            account,
            base_url: base_url.to_string(),
        }
    }

    pub async fn place_order(&self, params: Value) -> Result<Value> {
        let url = format!("{}/exchange", self.base_url);
        let response = self.client
            .post(&url)
            .json(&params)
            .send()
            .await?
            .json::<Value>()
            .await?;
        Ok(response)
    }

    pub async fn cancel_order(&self, params: Value) -> Result<Value> {
        let url = format!("{}/exchange", self.base_url);
        let response = self.client
            .post(&url)
            .json(&params)
            .send()
            .await?
            .json::<Value>()
            .await?;
        Ok(response)
    }
}

#[derive(Clone)]
pub struct Info {
    client: Client,
    base_url: String,
}

impl Info {
    pub async fn new(base_url: &str) -> Option<Self> {
        Some(Self {
            client: Client::new(),
            base_url: base_url.to_string(),
        })
    }

    pub async fn meta(&self) -> Result<Value> {
        let url = format!("{}/info", self.base_url);
        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "type": "meta"
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;
        Ok(response)
    }

    pub async fn user_state(&self, address: &str) -> Result<Value> {
        let url = format!("{}/info", self.base_url);
        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "type": "clearinghouseState",
                "user": address
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;
        Ok(response)
    }

    pub async fn open_orders(&self, address: &str) -> Result<Vec<Value>> {
        let url = format!("{}/info", self.base_url);
        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "type": "openOrders",
                "user": address
            }))
            .send()
            .await?
            .json::<Vec<Value>>()
            .await?;
        Ok(response)
    }

    pub async fn spot_user_state(&self, address: &str) -> Result<Value> {
        let url = format!("{}/info", self.base_url);
        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "type": "spotClearinghouseState",
                "user": address
            }))
            .send()
            .await?
            .json::<Value>()
            .await?;
        Ok(response)
    }
}