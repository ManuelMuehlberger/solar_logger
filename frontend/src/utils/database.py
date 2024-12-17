import sqlite3
import pandas as pd
from datetime import datetime, timedelta
import requests
import numpy as np
from config import BACKEND_URL, DB_PATH
import struct

def float16_to_float32(int16_val: int) -> float:
    """Convert a 16-bit integer representing an f16 to a Python float."""
    # Convert int16 to uint16 for proper bit manipulation
    uint16_val = int16_val & 0xFFFF
    packed = struct.pack('H', uint16_val)
    float16_val = np.frombuffer(packed, dtype=np.float16)[0]
    return float(float16_val)

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

def load_data(start_time=None, end_time=None, meter_name=None):
    """
    Load data from the database with optional filtering by time range and meter name.
    Args:
        start_time: Optional datetime for filtering data after this time
        end_time: Optional datetime for filtering data before this time
        meter_name: Optional string to filter data for a specific meter
    Returns:
        pandas.DataFrame with the requested data
    """
    conn = sqlite3.connect(DB_PATH)
    
    query = """
    SELECT 
        mn.name as meter_name,
        r.timestamp,
        r.total_power,
        r.import_power,
        r.export_power,
        r.total_kwh
    FROM meter_readings r
    JOIN meter_names mn ON r.meter_id = mn.meter_id
    WHERE 1=1
    """
    
    params = []
    
    if start_time:
        query += " AND r.timestamp >= ?"
        params.append(int(start_time.timestamp()))
    
    if end_time:
        query += " AND r.timestamp <= ?"
        params.append(int(end_time.timestamp()))
    
    if meter_name:
        query += " AND mn.name = ?"
        params.append(meter_name)
    
    query += " ORDER BY r.timestamp DESC"
    
    df = pd.read_sql_query(query, conn, params=params)
    
    # Convert timestamp from Unix timestamp to datetime
    df['timestamp'] = pd.to_datetime(df['timestamp'], unit='s')
    
    # Convert f16 power readings to f32
    for col in ['total_power', 'import_power', 'export_power']:
        df[col] = df[col].apply(float16_to_float32)
    
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
    
    # Calculate time differences for energy calculation
    df['time_diff'] = df['timestamp'].diff(-1).dt.total_seconds().abs() / 3600  # Convert to hours
    
    stats = {
        'total_import': (df['import_power'] * df['time_diff']).sum() / 1000,  # Convert W to kWh
        'total_export': (df['export_power'] * df['time_diff']).sum() / 1000,  # Convert W to kWh
        'peak_power': df['total_power'].abs().max(),
        'average_power': df['total_power'].mean()
    }
    
    return stats