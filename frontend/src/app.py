import streamlit as st
import pandas as pd
import plotly.express as px
import plotly.graph_objects as go
import sqlite3
from datetime import datetime, timedelta
import requests

# Constants
DB_PATH = "/Users/manu/Documents/Privat/Privat/solarmeter/db/solar_db.db"
BACKEND_URL = "http://localhost:8081"

# Page config
st.set_page_config(
    page_title="Solar Meter Dashboard",
    page_icon="☀️",
    layout="wide"
)

# Backend data fetching functions
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

# Sidebar
def render_sidebar():
    st.sidebar.title("Navigation")
    page = st.sidebar.radio("Select Page", ["Home", "Charts", "Info"])
    
    st.sidebar.markdown("---")
    st.sidebar.header("Backend Status")
    
    status = get_backend_status()
    if status:
        st.sidebar.success("Backend Connected")
        st.sidebar.metric("Database Size", f"{status['database_size_bytes'] / 1024 / 1024:.2f} MB")
        st.sidebar.metric("Total Records", f"{status['total_records']:,}")
    else:
        st.sidebar.error("Backend Not Connected")
    
    meters = get_meter_status()
    if meters:
        st.sidebar.markdown("---")
        st.sidebar.header("Active Meters")
        for meter in meters:
            with st.sidebar.expander(meter['meter_name']):
                st.write(f"Last Reading: {meter['last_power_reading']:.2f} W")
                st.write(f"Total Readings: {meter['total_readings']:,}")
                st.write(f"Last Update: {meter['last_reading_timestamp']}")
    
    return page

# Pages
def home_page():
    st.title("☀️ Solar Meter Dashboard")
    st.markdown("""
    Welcome to the Solar Meter Dashboard! This application helps you monitor and analyze your solar power generation and consumption.
    
    ### Features:
    - Real-time meter monitoring
    - Power generation analytics
    - Import/Export power analysis
    - Historical data visualization
    
    ### Getting Started:
    1. Use the sidebar to navigate between different pages
    2. Check the backend status in the sidebar
    3. View real-time meter readings
    4. Analyze historical data in the Charts section
    """)
    
    # Recent Data Preview
    st.header("Recent Readings")
    df = load_data(datetime.utcnow() - timedelta(hours=1))
    if not df.empty:
        fig = px.line(
            df,
            x='timestamp',
            y='total_power',
            color='meter_name',
            title="Power Output - Last Hour"
        )
        st.plotly_chart(fig, use_container_width=True)
    else:
        st.warning("No recent data available")

def charts_page():
    st.title("Power Analysis Charts")
    
    # Time range selector
    time_range = st.selectbox(
        "Select Time Range",
        ["Last Hour", "Last 24 Hours", "Last 7 Days", "Last 30 Days", "Custom"]
    )
    
    now = datetime.utcnow()
    if time_range == "Last Hour":
        start_time = now - timedelta(hours=1)
        end_time = now
    elif time_range == "Last 24 Hours":
        start_time = now - timedelta(days=1)
        end_time = now
    elif time_range == "Last 7 Days":
        start_time = now - timedelta(days=7)
        end_time = now
    elif time_range == "Last 30 Days":
        start_time = now - timedelta(days=30)
        end_time = now
    else:
        col1, col2 = st.columns(2)
        with col1:
            start_time = st.date_input("Start Date")
        with col2:
            end_time = st.date_input("End Date")
    
    df = load_data(start_time, end_time)
    
    if not df.empty:
        # Total Power Chart
        st.header("Total Power")
        fig_total = px.line(
            df,
            x='timestamp',
            y='total_power',
            color='meter_name',
            title="Total Power Over Time"
        )
        st.plotly_chart(fig_total, use_container_width=True)
        
        # Import vs Export
        col1, col2 = st.columns(2)
        with col1:
            fig_import = px.line(
                df,
                x='timestamp',
                y='import_power',
                color='meter_name',
                title="Import Power"
            )
            st.plotly_chart(fig_import)
        
        with col2:
            fig_export = px.line(
                df,
                x='timestamp',
                y='export_power',
                color='meter_name',
                title="Export Power"
            )
            st.plotly_chart(fig_export)
        
        # Daily Aggregates
        st.header("Daily Analysis")
        df['date'] = df['timestamp'].dt.date
        daily_stats = df.groupby(['date', 'meter_name']).agg({
            'total_power': ['mean', 'min', 'max'],
            'import_power': 'sum',
            'export_power': 'sum'
        }).reset_index()
        
        fig_daily = go.Figure()
        for meter in df['meter_name'].unique():
            meter_data = daily_stats[daily_stats['meter_name'] == meter]
            fig_daily.add_trace(go.Bar(
                name=f"{meter} - Import",
                x=meter_data['date'],
                y=meter_data['import_power']['sum'],
                offsetgroup=meter
            ))
            fig_daily.add_trace(go.Bar(
                name=f"{meter} - Export",
                x=meter_data['date'],
                y=-meter_data['export_power']['sum'],
                offsetgroup=meter
            ))
        
        fig_daily.update_layout(
            title="Daily Import/Export Balance",
            barmode='relative',
            height=400
        )
        st.plotly_chart(fig_daily, use_container_width=True)
        
        # Raw Data Table
        with st.expander("View Raw Data"):
            st.dataframe(df)
    else:
        st.warning("No data available for the selected time range")

def info_page():
    st.title("System Information")
    
    # Backend Information
    st.header("Backend Status")
    status = get_backend_status()
    if status:
        cols = st.columns(3)
        with cols[0]:
            st.metric("Database Size", f"{status['database_size_bytes'] / 1024 / 1024:.2f} MB")
        with cols[1]:
            st.metric("Total Records", f"{status['total_records']:,}")
        with cols[2]:
            st.metric("Active Meters", status['meters_count'])
            
        st.subheader("Database Details")
        st.code(f"Path: {status['database_path']}")
        
    # Meter Details
    st.header("Meter Information")
    meters = get_meter_status()
    if meters:
        for meter in meters:
            with st.expander(f"📊 {meter['meter_name']}"):
                col1, col2 = st.columns(2)
                with col1:
                    st.metric("Current Power", f"{meter['last_power_reading']:.2f} W")
                    st.metric("Total Readings", meter['total_readings'])
                with col2:
                    st.metric("Last Update", meter['last_reading_timestamp'])
    else:
        st.error("Could not fetch meter information")

# Main app
def main():
    page = render_sidebar()
    
    if page == "Home":
        home_page()
    elif page == "Charts":
        charts_page()
    else:
        info_page()

if __name__ == "__main__":
    main()