import sqlite3
import pandas as pd
from datetime import datetime, timedelta
import requests
from config import BACKEND_URL, DB_PATH

def get_backend_status():
    try:
        response = requests.get(f"{BACKEND_URL}/status")
        return response.json()
    except:
        return None

def get_meter_status():
    try:
        response = requests.get(f"{BACKEND_URL}/meters")
        return response.json()
    except:
        return None

def load_data(start_time=None, end_time=None):
    conn = sqlite3.connect(DB_PATH)
    
    query = """
        SELECT meter_name, timestamp, total_power, import_power, export_power, total_kwh
        FROM meter_readings
        WHERE 1=1
    """
    params = []
    
    if start_time:
        query += " AND timestamp >= ?"
        params.append(start_time.isoformat())
    if end_time:
        query += " AND timestamp <= ?"
        params.append(end_time.isoformat())
        
    query += " ORDER BY timestamp DESC"
    
    df = pd.read_sql_query(query, conn, params=params)
    df['timestamp'] = pd.to_datetime(df['timestamp'])
    conn.close()
    return df

def get_current_power_usage():
    meters = get_meter_status()
    if not meters:
        return 0
    
    total_power = sum(meter['last_power_reading'] for meter in meters if meter['last_power_reading'])
    return abs(total_power)  # Use abs() since power might be negative for export

def get_daily_stats():
    today = datetime.now().date()
    start_time = datetime.combine(today, datetime.min.time())
    
    df = load_data(start_time)
    if df.empty:
        return {
            'total_import': 0,
            'total_export': 0,
            'peak_power': 0,
            'average_power': 0
        }
    
    stats = {
        'total_import': df['import_power'].sum() / 3600,  # Convert to kWh
        'total_export': df['export_power'].sum() / 3600,  # Convert to kWh
        'peak_power': df['total_power'].abs().max(),
        'average_power': df['total_power'].mean()
    }
    
    return stats