import sqlite3
import json
from datetime import datetime
from collections import Counter
import html as html_module

def escape_html(text):
    """Escape HTML special characters to prevent XSS"""
    if not text:
        return ''
    return html_module.escape(str(text))

def generate_html_report(db_path, output_path):
    """Generate a beautiful HTML report from the database"""
    
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    cursor = conn.cursor()
    
    # Fetch data
    cursor.execute('SELECT COUNT(*) as count FROM endpoints')
    endpoint_count = cursor.fetchone()['count']
    
    cursor.execute('SELECT COUNT(*) as count FROM requests')
    request_count = cursor.fetchone()['count']
    
    cursor.execute('SELECT COUNT(*) as count FROM findings')
    finding_count = cursor.fetchone()['count']
    
    cursor.execute('''
        SELECT status_code, COUNT(*) as count 
        FROM responses 
        GROUP BY status_code 
        ORDER BY status_code
    ''')
    status_codes = cursor.fetchall()
    
    cursor.execute('''
        SELECT url, method, source, discovered_at 
        FROM endpoints 
        ORDER BY discovered_at
    ''')
    endpoints = cursor.fetchall()
    
    cursor.execute('''
        SELECT status_code, AVG(duration_ms) as avg_time
        FROM responses
        GROUP BY status_code
    ''')
    response_times = cursor.fetchall()
    
    cursor.execute('''
        SELECT url, COUNT(*) as count 
        FROM requests 
        GROUP BY url 
        ORDER BY count DESC 
        LIMIT 10
    ''')
    top_urls = cursor.fetchall()
    
    cursor.execute('''
        SELECT severity, COUNT(*) as count 
        FROM findings 
        GROUP BY severity
    ''')
    findings_by_severity = cursor.fetchall()
    
    # Build status code chart data
    status_chart_data = ', '.join([f"{{x: '{row['status_code']}', y: {row['count']}}}" for row in status_codes])
    
    # Build HTML
    html = f"""
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self' https://cdn.jsdelivr.net; style-src 'self' 'unsafe-inline';">
    <title>Bug Bounty Security Report</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js@3.9.1/dist/chart.min.js"></script>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }}
        
        .container {{
            max-width: 1400px;
            margin: 0 auto;
        }}
        
        header {{
            text-align: center;
            color: white;
            margin-bottom: 40px;
        }}
        
        h1 {{
            font-size: 2.5em;
            margin-bottom: 10px;
        }}
        
        .subtitle {{
            font-size: 1.1em;
            opacity: 0.9;
        }}
        
        .metrics {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 40px;
        }}
        
        .metric-card {{
            background: white;
            padding: 25px;
            border-radius: 12px;
            box-shadow: 0 4px 15px rgba(0,0,0,0.1);
            text-align: center;
            transition: transform 0.3s ease, box-shadow 0.3s ease;
        }}
        
        .metric-card:hover {{
            transform: translateY(-5px);
            box-shadow: 0 8px 25px rgba(0,0,0,0.15);
        }}
        
        .metric-number {{
            font-size: 2.5em;
            font-weight: 700;
            color: #667eea;
            margin: 10px 0;
        }}
        
        .metric-label {{
            color: #666;
            font-size: 0.95em;
            text-transform: uppercase;
            letter-spacing: 1px;
        }}
        
        .content {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(500px, 1fr));
            gap: 20px;
            margin-bottom: 40px;
        }}
        
        .card {{
            background: white;
            border-radius: 12px;
            padding: 25px;
            box-shadow: 0 4px 15px rgba(0,0,0,0.1);
        }}
        
        .card h2 {{
            color: #333;
            margin-bottom: 20px;
            font-size: 1.3em;
            border-bottom: 2px solid #667eea;
            padding-bottom: 10px;
        }}
        
        .chart-container {{
            position: relative;
            height: 300px;
            margin-bottom: 20px;
        }}
        
        table {{
            width: 100%;
            border-collapse: collapse;
        }}
        
        th {{
            background: #f5f5f5;
            padding: 12px;
            text-align: left;
            font-weight: 600;
            color: #333;
            border-bottom: 2px solid #ddd;
        }}
        
        td {{
            padding: 12px;
            border-bottom: 1px solid #eee;
        }}
        
        tr:hover {{
            background: #f9f9f9;
        }}
        
        .badge {{
            display: inline-block;
            padding: 4px 12px;
            border-radius: 20px;
            font-size: 0.85em;
            font-weight: 600;
        }}
        
        .badge-success {{
            background: #d4edda;
            color: #155724;
        }}
        
        .badge-warning {{
            background: #fff3cd;
            color: #856404;
        }}
        
        .badge-danger {{
            background: #f8d7da;
            color: #721c24;
        }}
        
        .badge-info {{
            background: #d1ecf1;
            color: #0c5460;
        }}
        
        .endpoint-method {{
            font-weight: 600;
            width: 50px;
        }}
        
        .method-get {{ color: #61affe; }}
        .method-post {{ color: #49cc90; }}
        .method-put {{ color: #fca130; }}
        .method-delete {{ color: #f93e3e; }}
        
        .full-width {{
            grid-column: 1 / -1;
        }}
        
        footer {{
            text-align: center;
            color: white;
            margin-top: 40px;
            opacity: 0.9;
        }}
        
        .severity-critical {{ color: #dc3545; font-weight: 700; }}
        .severity-high {{ color: #fd7e14; font-weight: 700; }}
        .severity-medium {{ color: #ffc107; font-weight: 700; }}
        .severity-low {{ color: #28a745; font-weight: 700; }}
        
        @media print {{
            body {{ background: white; }}
            .metric-card {{ page-break-inside: avoid; }}
            .card {{ page-break-inside: avoid; }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>🛡️ Security Assessment Report</h1>
            <p class="subtitle">Generated on {datetime.now().strftime('%B %d, %Y at %H:%M:%S')}</p>
        </header>
        
        <div class="metrics">
            <div class="metric-card">
                <div class="metric-label">Endpoints Discovered</div>
                <div class="metric-number">{endpoint_count}</div>
            </div>
            <div class="metric-card">
                <div class="metric-label">Requests Made</div>
                <div class="metric-number">{request_count}</div>
            </div>
            <div class="metric-card">
                <div class="metric-label">Vulnerabilities Found</div>
                <div class="metric-number">{finding_count}</div>
            </div>
            <div class="metric-card">
                <div class="metric-label">Response Statuses</div>
                <div class="metric-number">{len(status_codes)}</div>
            </div>
        </div>
        
        <div class="content">
            <div class="card">
                <h2>Response Status Distribution</h2>
                <div class="chart-container">
                    <canvas id="statusChart"></canvas>
                </div>
            </div>
            
            <div class="card">
                <h2>Response Time by Status Code</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Status Code</th>
                            <th>Avg Response Time</th>
                        </tr>
                    </thead>
                    <tbody>
"""
    
    for row in response_times:
        html += f"""
                        <tr>
                            <td><span class="badge badge-info">{escape_html(str(row['status_code']))}</span></td>
                            <td>{row['avg_time']:.0f}ms</td>
                        </tr>
"""
    
    html += f"""
                    </tbody>
                </table>
            </div>
        </div>
        
        <div class="content">
            <div class="card full-width">
                <h2>Discovered Endpoints</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Method</th>
                            <th>Endpoint</th>
                            <th>Source</th>
                            <th>Discovered</th>
                        </tr>
                    </thead>
                    <tbody>
"""
    
    for endpoint in endpoints:
        method = endpoint['method']
        method_class = f"method-{method.lower()}"
        html += f"""
                        <tr>
                            <td><span class="endpoint-method {method_class}">{escape_html(method)}</span></td>
                            <td><code>{escape_html(endpoint['url'])}</code></td>
                            <td><span class="badge badge-success">{escape_html(endpoint['source'])}</span></td>
                            <td>{escape_html(endpoint['discovered_at'][:10])}</td>
                        </tr>
"""
    
    html += f"""
                    </tbody>
                </table>
            </div>
        </div>
"""
    
    if finding_count > 0:
        html += f"""
        <div class="content">
            <div class="card full-width">
                <h2>🚨 Vulnerabilities Found</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Severity</th>
                            <th>Title</th>
                            <th>Description</th>
                            <th>Endpoint</th>
                        </tr>
                    </thead>
                    <tbody>
"""
        cursor.execute('''
            SELECT f.severity, f.title, f.description, e.url
            FROM findings f
            JOIN endpoints e ON f.endpoint_id = e.id
            ORDER BY f.severity DESC
        ''')
        
        for finding in cursor.fetchall():
            severity_class = f"severity-{finding['severity'].lower()}"
            html += f"""
                        <tr>
                            <td><span class="{severity_class}">{escape_html(finding['severity'].upper())}</span></td>
                            <td>{escape_html(finding['title'])}</td>
                            <td>{escape_html(finding['description'])}</td>
                            <td><code>{escape_html(finding['url'])}</code></td>
                        </tr>
"""
        
        html += """
                    </tbody>
                </table>
            </div>
        </div>
"""
    
    html += f"""
        <div class="content">
            <div class="card full-width">
                <h2>Top 10 Most Tested Endpoints</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Endpoint</th>
                            <th>Request Count</th>
                        </tr>
                    </thead>
                    <tbody>
"""
    
    for url_data in top_urls:
        html += f"""
                        <tr>
                            <td><code>{escape_html(url_data['url'])}</code></td>
                            <td><span class="badge badge-info">{url_data['count']} requests</span></td>
                        </tr>
"""
    
    html += f"""
                    </tbody>
                </table>
            </div>
        </div>
        
        <footer>
            <p>Generated by BB-Engine Security Assessment Tool</p>
            <p>Report Date: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}</p>
        </footer>
    </div>
    
    <script>
        const ctx = document.getElementById('statusChart').getContext('2d');
        const statusChart = new Chart(ctx, {{
            type: 'bar',
            data: {{
                datasets: [{{
                    label: 'Response Count',
                    data: [{status_chart_data}],
                    backgroundColor: [
                        '#667eea',
                        '#764ba2',
                        '#f093fb',
                        '#4facfe',
                        '#00f2fe'
                    ],
                    borderRadius: 5,
                    borderSkipped: false
                }}]
            }},
            options: {{
                indexAxis: 'x',
                responsive: true,
                maintainAspectRatio: false,
                plugins: {{
                    legend: {{
                        display: false
                    }}
                }},
                scales: {{
                    y: {{
                        beginAtZero: true,
                        grid: {{
                            display: true,
                            color: '#f0f0f0'
                        }}
                    }},
                    x: {{
                        grid: {{
                            display: false
                        }}
                    }}
                }}
            }}
        }});
    </script>
</body>
</html>
"""
    
    # Write to file
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write(html)
    
    conn.close()
    print(f"✅ Report generated: {output_path}")
    print(f"📊 Open in browser to view the beautiful dashboard!")

if __name__ == '__main__':
    generate_html_report('bb-engine.db', 'security_report.html')
