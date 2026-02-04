# 🚀 Quick Start: BB-Engine with Professional Reporting

## 5-Minute Quick Start

### 1. Build the Project

```bash
cargo build --release
```

### 2. Test with Automated Workflow

```bash
python test_workflow.py https://httpbin.org quick
```

This will:

- ✅ Discover endpoints
- ✅ Run basic fuzzing
- ✅ Generate beautiful report
- ✅ Open in browser automatically

### 3. View the Report

The HTML dashboard opens automatically with:

- 📊 Interactive charts
- 📈 Statistics
- 🎯 Discovered endpoints
- 🚨 Any vulnerabilities found

### 4. Export as PDF

In the browser: `File → Print → Save as PDF`

---

## For BugCrowd Testing

### 1. Find Public Program

- Go to bugcrowd.com/programs
- Filter by "Public"
- Choose a target (look for "no approval needed" tags)

### 2. Run Assessment

```bash
# Replace with actual target
python test_workflow.py https://your-target.com pipeline
```

### 3. Review Report

- Open `security_report.html` in browser
- Export to PDF if needed

### 4. Submit Findings

- Add PDF/HTML to BugCrowd submission
- Include clear description of issues
- Add steps to reproduce

---

## Basic Commands

### Discovery Only

```bash
python test_workflow.py https://example.com discover
```

### Fuzzing Only

```bash
python test_workflow.py https://example.com fuzz
```

### Full Pipeline

```bash
python test_workflow.py https://example.com pipeline
```

### Quick Test (Fastest)

```bash
python test_workflow.py https://example.com quick
```

### Manual Testing

```bash
# Discover endpoints
.\target\release\bb-engine.exe --config examples/config.yml discover \
  --target https://example.com \
  --wordlist examples/wordlist.txt

# Run fuzzing
.\target\release\bb-engine.exe --config examples/config.yml fuzz \
  --target https://example.com/api/users?id=1 \
  --mode sqli

# Generate report
python generate_report.py
```

---

## Report Types

### Interactive HTML Dashboard

```bash
python generate_report.py
```

- 🌐 Beautiful charts
- 📱 Mobile responsive
- 🎨 Professional design
- 📤 Shareable via email

### PDF Export

From HTML report: `Print → Save as PDF`

- 📄 Perfect for documentation
- 🖨️ Print-friendly
- 📋 Formal submissions

### Raw Data

- 📊 `bb-engine.db` - SQLite database
- 📋 Query directly with SQL
- 🔧 Export to custom formats

---

## Example: Complete Workflow

```bash
# 1. Build
cargo build --release

# 2. Test (pick mode)
python test_workflow.py https://example.com pipeline

# 3. Wait for completion

# 4. Report opens automatically (or manually open)
start security_report.html

# 5. Export to PDF
# Browser: Ctrl+P → Save as PDF

# 6. Submit to BugCrowd
```

---

## Customization

### Change Wordlist

Edit `examples/wordlist.txt` to add custom entries:

```
admin
api
/admin/login
/api/users
custom-path
```

### Change Config

Edit `examples/config.yml`:

```yaml
http:
  rate_limit: 20  # Higher = faster but more aggressive
  timeout: 60     # Higher = wait longer for responses
  max_concurrent: 20  # Higher = more parallel requests
```

### Change Workflow

Edit `examples/workflow.yaml`:

```yaml
stages:
  - name: "Discovery"
    stage_type:
      type: Discovery
    config:
      wordlist: "examples/wordlist.txt"
      max_depth: 3
```

---

## Troubleshooting

### Report is empty

- Ensure assessment completed: check console for "completed successfully"
- Run: `python generate_report.py` again
- Check `bb-engine.db` exists: `ls bb-engine.db`

### Assessment running slowly

- Increase `max_concurrent` in config.yml
- Decrease `wordlist.txt` size for faster discovery
- Use `quick` mode instead of `pipeline`

### Port 8080 already in use

The HTTP server uses port 8080. If busy:

```bash
python -m http.server 8081
# Then visit: http://localhost:8081/security_report.html
```

### Python not found

Ensure Python 3.7+ installed:

```bash
python --version
```

---

## Key Files

```
bb-engine/
├── generate_report.py      # Generate beautiful HTML reports
├── test_workflow.py        # Automated testing workflow
├── BUGCROWD_GUIDE.md      # Complete BugCrowd guide
├── REPORTING_GUIDE.md     # Report system documentation
├── examples/
│   ├── config.yml         # Configuration (rate limits, etc.)
│   ├── wordlist.txt       # 553+ endpoints to test
│   ├── workflow.yaml      # Testing pipeline definition
│   ├── patterns.yml       # Pattern matching rules
│   └── patterns.yaml      # Pattern matching rules
├── security_report.html   # Generated report (after testing)
├── bb-engine.db          # Test results database
└── target/release/bb-engine.exe  # Compiled binary
```

---

## Next Steps

1. ✅ Build the project
2. ✅ Run `test_workflow.py` with a test target
3. ✅ View the beautiful report
4. ✅ Find a BugCrowd public program
5. ✅ Run real assessment
6. ✅ Submit findings!

---

**Happy Hunting! 🎯**

For more details:

- 📖 README.md - Full documentation
- 🐛 BUGCROWD_GUIDE.md - BugCrowd specific guide
- 📊 REPORTING_GUIDE.md - Report system guide
