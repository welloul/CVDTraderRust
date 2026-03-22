# AWS Deployment Guide (Amazon Linux)

This guide provides step-by-step instructions for deploying the **CVD Trader Bot** on an AWS EC2 instance running **Amazon Linux 2023** or **Amazon Linux 2**.

## 1. Prerequisites

### Recommended Instance Type
- **Minimum**: `t3.medium` (2 vCPU, 4GB RAM)
- **Optimal (for low latency)**: `c6i.large` or `c7g.medium` (Compute optimized)

### Security Group (Firewall)
In the AWS Console, ensure your Security Group allows:
- **SSH (22)**: For your IP.
- **Custom TCP (8000)**: For API access and monitoring (restrict to your IP or VPN).

---

## 2. System Preparation

SSH into your instance and install the required system dependencies:

```bash
# Update system
sudo dnf update -y  # Use 'yum' if on Amazon Linux 2

# Install development tools and libraries
sudo dnf groupinstall "Development Tools" -y
sudo dnf install -y openssl-devel sqlite-devel
```

---

## 3. Install Rust

Install the latest stable version of Rust using `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version
```

---

## 4. Clone and Build

```bash
# Clone the repository (Replace with your actual repo URL)
git clone https://github.com/welloul/CVDTraderRust
cd cvd-trader-rust

# Build the production binary
# LTO and abort-on-panic are enabled in Cargo.toml for performance
cargo build --release
```

The binary will be located at `./target/release/cvd_trader_rust`.

---

## 5. Configuration

Create your production `config.toml` in the project root:

```toml
[execution]
mode = "dryrun" # Change to "real" when ready
default_slippage_pct = 0.001

[strategy]
lookback = 20
cvd_exhaustion_ratio = 0.7
cvd_absorption_pctile = 0.9
fixed_fee_rate = 0.0003

[general]
target_coins = ["SOL", "ZEC", "HYPE", "XMR", "LINK", "XLM", "AVAX", "TON", "TAO"]
```

If using real execution, also ensure your `.env` file (if used) or environment variables for API keys are set.

---

## 6. Run as a Service (Systemd)

To ensure the bot survives crashes and restarts automatically on reboot, create a systemd unit file.

```bash
sudo nano /etc/systemd/system/cvd-trader.service
```

Paste the following configuration (adjust paths as necessary):

```ini
[Unit]
Description=CVD Trader Bot Rust
After=network.target

[Service]
Type=simple
User=ec2-user
WorkingDirectory=/home/ec2-user/CVDTraderRust
ExecStart=/home/ec2-user/CVDTraderRust/target/release/cvd_trader_rust
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

**Enable and start the service:**

```bash
sudo systemctl daemon-reload
sudo systemctl enable cvd-trader
sudo systemctl start cvd-trader
```

---

## 7. Monitoring

Check the status and logs:

```bash
# Check service status
sudo systemctl status cvd-trader

# Follow live logs
journalctl -u cvd-trader -f
```

**Access the API:**
Verify the bot is responding from your local machine:
`curl http://<your-ec2-ip>:8000/status`

---

## 8. Performance Tuning (Advanced)

### Network Latency
Since the bot connects to `api.hyperliquid.xyz`, deploying in **AWS Tokyo (ap-northeast-1)** or **AWS Ireland (eu-west-1)** may provide better latency depending on where the exchange servers are colocated.

### Memory Monitoring
The SQLite database stores logs and trades. Monitor disk usage in `backend/data/` if running for extended periods.
