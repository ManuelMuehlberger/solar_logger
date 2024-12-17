import sqlite3
import pandas as pd
from datetime import datetime
import requests
from utils.config import BACKEND_URL, DB_PATH, get_timezone
import numpy as np
from datetime import timezone

def float16_to_float32(int16_val: int) -> float:
    """Convert a 16-bit integer representing an f16 to a Python float."""
    uint16_val = int16_val & 0xFFFF
    return float(np.frombuffer(np.array([uint16_val], dtype='uint16').tobytes(), dtype=np.float16)[0])

def get_backend_status():
    try:
        response = requests.get(f"{BACKEND_URL}/status")
        if response.status_code == 200:
            return response.json()
    except:
        pass
    return None

def get_meter_status():
    try:
        response = requests.get(f"{BACKEND_URL}/meters")
        if response.status_code == 200:
            return response.json()
    except:
        pass
    return None

def to_unix_timestamp(dt):
    """Convert datetime to Unix timestamp, handling timezone-aware and naive datetimes"""
    if dt is None:
        return None
    if dt.tzinfo is None:
        # If naive, assume it's in configured timezone
        dt = dt.replace(tzinfo=get_timezone())
    return int(dt.timestamp())

def load_data(start_time=None, end_time=None, meter_name=None):
    """
    Load data from the database with optional filtering by time range and meter name.
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
        # Ensure start_time is in UTC
        if start_time.tzinfo is not None:
            start_time = start_time.astimezone(timezone.utc)
        unix_start = int(start_time.timestamp())
        query += " AND r.timestamp >= ?"
        params.append(unix_start)
    
    if end_time:
        # Ensure end_time is in UTC
        if end_time.tzinfo is not None:
            end_time = end_time.astimezone(timezone.utc)
        unix_end = int(end_time.timestamp())
        query += " AND r.timestamp <= ?"
        params.append(unix_end)
    
    if meter_name:
        query += " AND mn.name = ?"
        params.append(meter_name)
    
    query += " ORDER BY r.timestamp DESC"
    
    try:
        df = pd.read_sql_query(query, conn, params=params)
        
        if not df.empty:
            # First convert Unix timestamps to UTC datetime
            df['timestamp'] = pd.to_datetime(df['timestamp'], unit='s', utc=True)
            
            # Now convert to configured timezone
            configured_tz = get_timezone()
            df['timestamp'] = df['timestamp'].dt.tz_convert(configured_tz)
            
            # Convert f16 stored power values to f32
            for col in ['total_power', 'import_power', 'export_power']:
                df[col] = df[col].apply(float16_to_float32)
        
        return df
    except Exception as e:
        print(f"Error loading data: {e}")
        return pd.DataFrame()
    finally:
        conn.close()

def get_current_power_usage():
    """Get the current total power usage across all meters"""
    meters = get_meter_status()
    if not meters:
        return 0.0
    try:
        return abs(sum(float(meter.get('last_power_reading', 0)) for meter in meters))
    except:
        return 0.0

def get_daily_stats():
    """Calculate daily statistics for power usage"""
    configured_tz = get_timezone()
    today = datetime.now(configured_tz).replace(hour=0, minute=0, second=0, microsecond=0)
    df = load_data(start_time=today)
    
    if df.empty:
        return {
            'total_import': 0.0,
            'total_export': 0.0,
            'peak_power': 0.0,
            'average_power': 0.0
        }
    
    # Calculate time differences for energy calculation
    df['time_diff'] = df['timestamp'].diff(-1).dt.total_seconds().abs() / 3600  # Convert to hours
    
    stats = {
        'total_import': (df['import_power'] * df['time_diff']).sum() / 1000,  # Convert Wh to kWh
        'total_export': (df['export_power'] * df['time_diff']).sum() / 1000,  # Convert Wh to kWh
        'peak_power': df['total_power'].abs().max(),
        'average_power': df['total_power'].mean()
    }
    
    return stats