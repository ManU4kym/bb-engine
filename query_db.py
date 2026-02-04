import sqlite3
import json

conn = sqlite3.connect('bb-engine.db')
conn.row_factory = sqlite3.Row
cursor = conn.cursor()

print("=" * 80)
print("DATABASE SUMMARY")
print("=" * 80)

# Table counts
tables = ['endpoints', 'requests', 'responses', 'findings']
for table in tables:
    cursor.execute(f'SELECT COUNT(*) as count FROM {table}')
    count = cursor.fetchone()['count']
    print(f"  {table:15} : {count:5} rows")

print("\n" + "=" * 80)
print("ENDPOINTS (Discovered)")
print("=" * 80)
cursor.execute('SELECT url, method, source, discovered_at FROM endpoints ORDER BY discovered_at')
for row in cursor.fetchall():
    print(f"  {row['method']:6} {row['url']:50} [{row['source']:10}]")

print("\n" + "=" * 80)
print("TOP REQUESTS BY URL")
print("=" * 80)
cursor.execute('''
    SELECT url, COUNT(*) as count 
    FROM requests 
    GROUP BY url 
    ORDER BY count DESC 
    LIMIT 10
''')
for row in cursor.fetchall():
    print(f"  {row['count']:3}x {row['url'][:70]}")

print("\n" + "=" * 80)
print("RESPONSE STATUS CODES")
print("=" * 80)
cursor.execute('''
    SELECT status_code, COUNT(*) as count 
    FROM responses 
    GROUP BY status_code 
    ORDER BY status_code
''')
for row in cursor.fetchall():
    print(f"  {row['status_code']:3} : {row['count']:3} responses")

print("\n" + "=" * 80)
print("SAMPLE RESPONSES (First 5)")
print("=" * 80)
cursor.execute('''
    SELECT r.id, r.status_code, r.duration_ms, LENGTH(r.body) as body_size
    FROM responses r
    LIMIT 5
''')
for row in cursor.fetchall():
    print(f"  ID: {row['id'][:20]:20} | Status: {row['status_code']:3} | Time: {row['duration_ms']:4}ms | Size: {row['body_size']:8} bytes")

print("\n" + "=" * 80)

conn.close()
