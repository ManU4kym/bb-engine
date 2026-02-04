#!/usr/bin/env python3
"""
Automated BB-Engine Testing & Reporting Workflow
Handles: Discovery -> Fuzzing -> Report Generation -> Export
"""

import sys
import subprocess
import webbrowser
import time
import random
from pathlib import Path
from datetime import datetime
from generate_report import generate_html_report

# Friendly spinners and short messages (kept simple to avoid encoding issues)
SPINNERS = {
    "discovering": ["🔍 .", "🔍 ..", "🔍 ...", "🔍 searching"],
    "fuzzing": ["💥 .", "💥 ..", "💥 ...", "💥 pwning"],
    "matching": ["🎯 .", "🎯 ..", "🎯 ...", "🎯 matching"],
}

VIBES = [
    "no cap, still scanning bestie",
    "we're cooking with gas",
    "just vibing, finding those vulns",
    "the grind never stops",
    "it's giving security findings",
]


def run_command_with_progress(cmd, description, update_every=20):
    """Run a command and show lightweight progress without hiding prompts."""
    print("\n" + "=" * 70)
    print(f"📍 {description}")
    print("=" * 70)

    mode = "discovering" if "discover" in cmd else "fuzzing" if "fuzz" in cmd else "matching"
    spinner = SPINNERS.get(mode, SPINNERS["discovering"])
    vibe_msg = random.choice(VIBES)
    start_time = time.time()

    print(f"\n🚀 Initializing {description.lower()}...\n")

    try:
        proc = subprocess.Popen(
            cmd,
            shell=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            encoding='utf-8',
            errors='replace',
            bufsize=1,
        )

        spin_idx = 0
        line_count = 0

        # Stream output: show important findings and periodic spinner
        for raw_line in proc.stdout:
            line = raw_line.rstrip()
            line_count += 1

            # Show important log lines immediately (Found, INFO, ERROR, WARN)
            if any(key in line for key in ["Found:", "INFO", "ERROR", "WARN"]):
                print(line)
            # Show spinner update every N lines
            elif line_count % update_every == 0:
                spin_msg = spinner[spin_idx % len(spinner)]
                elapsed = time.time() - start_time
                print(f"⏱ {int(elapsed)}s | {spin_msg} | {vibe_msg}")
                spin_idx += 1

        proc.wait()

        # Ensure we end on a clean line so input() prompts show correctly
        print()

        if proc.returncode != 0:
            print(f"❌ Command failed with exit code {proc.returncode}")
            return False

        elapsed = time.time() - start_time
        print(f"✅ {description} completed in {elapsed:.1f}s!\n")
        return True

    except Exception as e:
        print(f"\n❌ Error: {e}")
        return False
def main():
    if len(sys.argv) < 2:
        print("""
Usage: python test_workflow.py <target_url> [mode]

Modes:
  discover    - Endpoint discovery only
  fuzz        - Fuzzing only
  pipeline    - Full pipeline (default)
  quick       - Quick discovery + shallow fuzzing

Examples:
  python test_workflow.py https://example.com pipeline
  python test_workflow.py https://example.com quick
""")
        sys.exit(1)

    target = sys.argv[1]
    mode = sys.argv[2] if len(sys.argv) > 2 else "pipeline"

    print(f"\n== BB-Engine: Security Assessment Workflow ==")
    print(f"Target: {target}")
    print(f"Mode: {mode}")
    print(f"Started: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")

    # Confirm database reset
    db_path = Path("bb-engine.db")
    if db_path.exists():
        resp = input("\nExisting database found. Type 'y' to delete and start fresh, or press Enter to keep it: ")
        if resp.strip().lower() == 'y':
            try:
                db_path.unlink()
                print("Old database deleted")
            except PermissionError:
                print("⚠️  Database is locked by another process. Keeping existing data.")

    # locate binary
    binary = Path("target/release/bb-engine.exe").resolve()
    if not binary.exists():
        print(f"Error: binary not found at {binary}")
        sys.exit(1)

    # Run discovery
    if mode in ("discover", "pipeline", "quick"):
        depth = 1 if mode == "quick" else 2
        run_command_with_progress(
            f'"{binary}" --config examples/config.yml discover --target {target} --wordlist examples/wordlist.txt --depth {depth}',
            f"Endpoint Discovery (depth={depth})"
        )

    # Run fuzz modes
    if mode in ("fuzz", "pipeline", "quick"):
        fuzz_list = ["sqli"] if mode == "quick" else ["sqli", "xss", "idor", "lfi"]
        for fm in fuzz_list:
            desc = fm.upper() + " fuzzing"
            run_command_with_progress(
                f'"{binary}" --config examples/config.yml fuzz --target {target} --mode {fm}',
                desc
            )

    # Generate report
    print("\n" + "=" * 60)
    print("📊 Generating report")
    print("=" * 60 + "\n")

    try:
        generate_html_report('bb-engine.db', 'security_report.html')
        print("Report generated: security_report.html")
        report_path = Path('security_report.html').resolve()
        open_resp = input("\nOpen report in browser? (y/n): ")
        if open_resp.strip().lower() == 'y':
            webbrowser.open(f'file://{report_path}')
            print("Opened in browser")
    except Exception as e:
        print(f"Error generating report: {e}")
        sys.exit(1)

    print("\nAssessment complete. Review the HTML report and submit findings if needed.")


if __name__ == '__main__':
    main()                    
    # End of script
if __name__ == '__main__':
    main()
