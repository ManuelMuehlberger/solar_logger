import streamlit as st
import plotly.graph_objects as go
from datetime import datetime, timedelta
import pandas as pd
from utils.database import get_backend_status, get_meter_status, load_data
from components.sidebar import render_sidebar

st.set_page_config(page_title="System Information", page_icon="ℹ️", layout="wide")

render_sidebar()

def render_system_overview():
    st.title("ℹ️ System Information")
    
    status = get_backend_status()
    if not status:
        st.error("Cannot connect to backend system")
        return
    
    # System metrics
    col1, col2, col3, col4 = st.columns(4)
    
    with col1:
        st.metric(
            "Database Size",
            f"{status['database_size_bytes'] / 1024 / 1024:.2f} MB",
        )
    
    with col2:
        st.metric(
            "Total Records",
            f"{status['total_records']:,}",
        )
    
    with col3:
        st.metric(
            "Active Meters",
            status['meters_count'],
        )
    
    with col4:
        uptime = timedelta(seconds=status.get('uptime_seconds', 0))
        st.metric(
            "System Uptime",
            f"{uptime.days}d {uptime.seconds//3600}h {(uptime.seconds//60)%60}m"
        )

def render_meter_details():
    st.header("Meter Details")
    
    meters = get_meter_status()
    if not meters:
        st.warning("No meter information available")
        return
    
    # Create expandable sections for each meter
    for meter in meters:
        with st.expander(f"📊 {meter['meter_name']}", expanded=True):
            col1, col2, col3 = st.columns(3)
            
            with col1:
                st.metric(
                    "Current Power",
                    f"{meter['last_power_reading']:.2f} W"
                )
            
            with col2:
                st.metric(
                    "Total Readings",
                    f"{meter['total_readings']:,}"
                )
            
            with col3:
                last_update = datetime.fromisoformat(
                    meter['last_reading_timestamp'].replace('Z', '+00:00')
                )
                st.metric(
                    "Last Update",
                    last_update.strftime("%H:%M:%S")
                )
            
            # Add meter-specific analytics
            df = load_data(
                datetime.utcnow() - timedelta(hours=24),
                datetime.utcnow(),
                meter['meter_name']
            )
            
            if not df.empty:
                col1, col2 = st.columns(2)
                
                with col1:
                    # Power stability gauge
                    power_std = df['total_power'].std()
                    max_power = df['total_power'].max()
                    stability = max(0, min(100, 100 * (1 - power_std / max_power)))
                    
                    fig = go.Figure(go.Indicator(
                        mode = "gauge+number",
                        value = stability,
                        domain = {'x': [0, 1], 'y': [0, 1]},
                        title = {'text': "Power Stability (%)"},
                        gauge = {
                            'axis': {'range': [0, 100]},
                            'steps': [
                                {'range': [0, 50], 'color': "lightgray"},
                                {'range': [50, 80], 'color': "gray"},
                                {'range': [80, 100], 'color': "darkgreen"}
                            ],
                            'threshold': {
                                'line': {'color': "red", 'width': 4},
                                'thickness': 0.75,
                                'value': stability
                            }
                        }
                    ))
                    
                    st.plotly_chart(fig, use_container_width=True)
                
                with col2:
                    # Reading frequency analysis
                    df['time_diff'] = df['timestamp'].diff().dt.total_seconds()
                    avg_interval = df['time_diff'].mean()
                    expected_interval = 60  # assuming 1-minute intervals
                    reliability = min(100, 100 * (expected_interval / avg_interval))
                    
                    fig = go.Figure(go.Indicator(
                        mode = "gauge+number",
                        value = reliability,
                        domain = {'x': [0, 1], 'y': [0, 1]},
                        title = {'text': "Reading Reliability (%)"},
                        gauge = {
                            'axis': {'range': [0, 100]},
                            'steps': [
                                {'range': [0, 60], 'color': "lightgray"},
                                {'range': [60, 85], 'color': "gray"},
                                {'range': [85, 100], 'color': "darkgreen"}
                            ],
                            'threshold': {
                                'line': {'color': "red", 'width': 4},
                                'thickness': 0.75,
                                'value': reliability
                            }
                        }
                    ))
                    
                    st.plotly_chart(fig, use_container_width=True)

def render_database_stats():
    st.header("Database Statistics")
    
    # Load last 30 days of data for analysis
    df = load_data(
        datetime.utcnow() - timedelta(days=30),
        datetime.utcnow()
    )
    
    if df.empty:
        st.warning("No data available for analysis")
        return
    
    col1, col2 = st.columns(2)
    
    with col1:
        # Data growth over time
        df['date'] = df['timestamp'].dt.date
        daily_counts = df.groupby('date').size().reset_index(name='count')
        
        fig = go.Figure()
        fig.add_trace(go.Scatter(
            x=daily_counts['date'],
            y=daily_counts['count'].cumsum(),
            mode='lines+markers',
            name='Cumulative Records'
        ))
        
        fig.update_layout(
            title="Database Growth Over Time",
            xaxis_title="Date",
            yaxis_title="Total Records",
            height=400
        )
        
        st.plotly_chart(fig, use_container_width=True)
    
    with col2:
        # Reading frequency distribution
        df['hour'] = df['timestamp'].dt.hour
        hourly_counts = df.groupby('hour').size().reset_index(name='count')
        
        fig = go.Figure()
        fig.add_trace(go.Bar(
            x=hourly_counts['hour'],
            y=hourly_counts['count'],
            name='Records per Hour'
        ))
        
        fig.update_layout(
            title="Reading Frequency by Hour",
            xaxis_title="Hour of Day",
            yaxis_title="Number of Readings",
            height=400
        )
        
        st.plotly_chart(fig, use_container_width=True)

def main():
    render_system_overview()
    st.markdown("---")
    render_meter_details()
    st.markdown("---")
    render_database_stats()
    
    # Add system configuration details
    st.markdown("---")
    st.header("System Configuration")
    
    status = get_backend_status()