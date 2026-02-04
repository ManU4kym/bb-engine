# Bug Bounty Automation Engine (bb-engine)

A powerful, production-ready automation engine for bug bounty hunting built in Rust. Designed for accuracy, speed, and reducing false positives.

## Features

- 🔍 **Endpoint Discovery**: Crawling, wordlist-based brute forcing, JavaScript analysis
- 🎯 **Smart Fuzzing**: SQLi, XSS, IDOR, LFI, SSRF, Command Injection with automatic payload generation
- 📊 **Pattern Matching**: Regex-based detection for secrets, API keys, errors, and vulnerabilities
- 🔄 **Pipeline Orchestration**: YAML-based workflow configuration for complex testing scenarios
- 💾 **Data Management**: SQLite storage with structured findings and evidence preservation
- ⚡ **High Performance**: Async I/O with configurable concurrency and rate limiting
- 🎨 **Professional Reports**: Beautiful HTML dashboards perfect for sharing with stakeholders
- 📱 **Export Options**: JSON, CSV, Markdown report generation + interactive HTML dashboards

## Architecture

```bash
bb-engine/
├── src/
│   ├── main.rs              # Entry point
│   ├── cli.rs               # CLI interface
│   ├── config.rs            # Configuration management
│   └── modules/
│       ├── discovery.rs     # Endpoint discovery
│       ├── fuzzer.rs        # Fuzzing engine
│       ├── pattern.rs       # Pattern matching
│       ├── pipeline.rs      # Workflow orchestration
│       ├── http_client.rs   # HTTP client wrapper
│       ├── storage.rs       # Database operations
│       └── types.rs         # Shared data structures
├── examples/
│   ├── config.yaml          # Example configuration
│   ├── patterns.yaml        # Pattern definitions
│   ├── workflow.yaml        # Workflow definition
│   └── wordlist.txt         # Endpoint wordlist
└── Cargo.toml
```

## Installation

### Prerequisites

- Rust 1.70 or higher
- SQLite3

### Build from Source

```bash
git clone <repo-url>
cd bb-engine
cargo build --release
```

The binary will be in `target/release/bb-engine`.

## Quick Start

### 1. Endpoint Discovery

Discover endpoints by crawling and brute forcing:

```bash
bb-engine discover \
  --target https://example.com \
  --wordlist examples/wordlist.txt \
  --depth 3
```

### 2. Run Fuzzing

Fuzz discovered endpoints for SQL injection:

```bash
bb-engine fuzz \
  --target https://example.com/api/user?id=1 \
  --mode sqli
```

Available fuzzing modes:

- `sqli` - SQL Injection
- `xss` - Cross-Site Scripting
- `idor` - Insecure Direct Object Reference
- `lfi` - Local File Inclusion
- `ssrf` - Server-Side Request Forgery
- `cmd_injection` - Command Injection

### 3. Pattern Matching

Search for secrets and vulnerabilities in saved responses:

```bash
bb-engine match \
  --database bb-engine.db \
  --patterns examples/patterns.yaml
```

### 4. Full Pipeline

Run a complete workflow:

```bash
bb-engine --config examples/config.yaml pipeline \
  --target https://example.com \
  --workflow examples/workflow.yaml
```

### 5. Export Findings

Export results to various formats:

```bash
# JSON export
bb-engine export \
  --database bb-engine.db \
  --format json \
  --output findings.json

# Markdown report
bb-engine export \
  --database bb-engine.db \
  --format markdown \
  --output report.md

# Beautiful HTML Dashboard (Interactive)
python generate_report.py
# Opens: security_report.html in browser
```

## 🎨 Professional Reporting System

Generate beautiful, shareable reports perfect for stakeholders and bug bounty submissions:

### Automated Workflow

```bash
# Run complete assessment and generate report
python test_workflow.py https://example.com pipeline
```

### Manual Report Generation

```bash
# After running any bb-engine command
python generate_report.py
```

### Report Features

- ✅ Interactive charts and visualizations
- ✅ Professional Notion-style design
- ✅ Mobile-responsive layout
- ✅ Print-to-PDF export
- ✅ Perfect for non-technical stakeholders
- ✅ No special software required (works in any browser)
- ✅ Shareable HTML file (email, web hosting, etc.)

## Configuration

Create a `config.yaml` file:

```yaml
http:
  max_concurrent: 10
  timeout: 30
  rate_limit: 10
  user_agent: "bb-engine/0.1.0"
  follow_redirects: true

discovery:
  extensions:
    - php
    - asp
    - json
  success_codes:
    - 200
    - 301

fuzzing:
  auto_generate: true
  payloads_per_param: 50
  delay_ms: 100
  similarity_threshold: 0.85

storage:
  db_path: "bb-engine.db"
  auto_save_interval: 60
```

## Pattern Matching

Define custom patterns in YAML:

```yaml
patterns:
  - name: "AWS Access Key"
    description: "AWS access key detected"
    regex: "AKIA[0-9A-Z]{16}"
    severity: Critical
    match_type: Body
    context_lines: 2

  - name: "SQL Error"
    description: "SQL error indicating SQLi vulnerability"
    regex: "(SQL syntax.*?MySQL)"
    severity: High
    match_type: Body
    context_lines: 3
```

Severity levels: `Critical`, `High`, `Medium`, `Low`, `Info`

Match types: `Body`, `Headers`, `Both`, `Request`

## Workflows

Create complex testing workflows:

```yaml
name: "Full Security Assessment"
description: "Complete automated security test"

stages:
  - name: "Discovery"
    stage_type: Discovery
    config:
      wordlist: "wordlist.txt"
      max_depth: 3

  - name: "SQL Injection"
    stage_type: Fuzzing
    config:
      fuzzing_mode: "sqli"

  - name: "Find Secrets"
    stage_type: PatternMatching
    config:
      pattern_file: "patterns.yaml"
```

## False Positive Reduction

The fuzzer uses multiple techniques to reduce false positives:

1. **Baseline Comparison**: Establishes normal response patterns
2. **Response Similarity**: Uses Levenshtein distance to detect meaningful changes
3. **Timing Analysis**: Identifies time-based vulnerabilities (blind SQLi, etc.)
4. **Error Pattern Detection**: Looks for specific error indicators
5. **Content Analysis**: Checks if payloads are reflected or executed

Configure sensitivity in `config.yaml`:

```yaml
fuzzing:
  similarity_threshold: 0.85  # 0.0 = all different, 1.0 = identical
```

## Database Schema

SQLite database structure:

- `endpoints` - Discovered endpoints with metadata
- `requests` - HTTP requests made during testing
- `responses` - HTTP responses received
- `findings` - Identified vulnerabilities with evidence

All findings include complete request/response pairs for verification.

## Advanced Usage

### Custom Headers

Add authentication or custom headers:

```yaml
http:
  headers:
    - ["Authorization", "Bearer YOUR_TOKEN"]
    - ["X-API-Key", "your-api-key"]
```

### Rate Limiting

Control request rate to avoid overwhelming targets:

```yaml
http:
  rate_limit: 10  # requests per second
  max_concurrent: 5  # concurrent requests
```

### Custom Payloads

Use your own payload file:

```bash
bb-engine fuzz \
  --target https://example.com/search?q=test \
  --mode custom \
  --payloads my-payloads.txt
```

## Performance Tips

1. **Adjust concurrency**: Increase `max_concurrent` for faster scanning
2. **Use quality wordlists**: The included wordlist covers 400+ common endpoints, API paths, admin panels, and configuration files. For production testing, consider:
   - Domain-specific wordlists (e.g., SecLists repository)
   - Historical data from similar targets
   - Generated patterns based on application framework
3. **Tune rate limits**: Respect target infrastructure limits
4. **Filter endpoints**: Focus fuzzing on interesting endpoints only

## Security Considerations

⚠️ **Important**: Only test targets you have permission to test.

- Unauthorized testing is illegal
- Always get written permission
- Respect rate limits and scope
- Review bug bounty program rules

## Troubleshooting

### "Too many open files" error

Increase system limits:

```bash
ulimit -n 4096
```

### Slow fuzzing

- Reduce `payloads_per_param` in config
- Increase `delay_ms` if target is rate limiting
- Check network connectivity

### Database locked

Only one process can write to the database at a time. Ensure no other instances are running.

## Contributing

Contributions welcome! Areas for improvement:

- Additional fuzzing modes
- Machine learning for false positive reduction
- Browser automation for JavaScript-heavy apps
- Integration with other security tools
- Additional export formats

## License

MIT License - see LICENSE file for details

## Disclaimer

This tool is for authorized security testing only. The authors are not responsible for misuse or damage caused by this program.

## Roadmap

- [ ] Browser automation with headless Chrome
- [ ] ML-based anomaly detection
- [ ] Collaborative features for teams
- [ ] Integration with popular bug bounty platforms
- [ ] GraphQL and WebSocket support
- [ ] Report generation with screenshots
- [ ] Distributed scanning across multiple machines
