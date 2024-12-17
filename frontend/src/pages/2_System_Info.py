import streamlit as st
import plotly.graph_objects as go
from datetime import datetime, timedelta
import pandas as pd
from utils.database import get_backend_status, get_meter_status, load_data
from components.sidebar import render_sidebar

# Page config
st.set_page_config(
    page_title="System Information",
    page_icon="ℹ️",
    layout="wide"
)

# Render the persistent sidebar
render_sidebar()

def format_timestamp(timestamp):
    """Convert Unix timestamp to formatted datetime string"""
    try:
        if timestamp is None:
            return None
            
        # If timestamp is a string that looks like a number, convert it
        if isinstance(timestamp, str):
            try:
                timestamp = int(float(timestamp))
            except ValueError:
                return None
                
        # Convert Unix timestamp to datetime
        if isinstance(timestamp, (int, float)):
            return datetime.fromtimestamp(timestamp).strftime('%Y-%m-%d %H:%M:%S')
            
        return None
    except Exception as e:
        print(f"Error formatting timestamp {timestamp}: {e}")
        return None

def render_system_overview():
    """Render the system overview section"""
    st.title("ℹ️ System Information")
    
    try:
        status = get_backend_status()
        if not status:
            st.error("Cannot connect to backend system")
            return
        
        # System metrics
        col1, col2, col3, col4 = st.columns(4)
        
        with col1:
            st.metric(
                "Database Size",
                f"{status.get('database_size_bytes', 0) / 1024 / 1024:.2f} MB",
            )
        
        with col2:
            st.metric(
                "Total Records",
                f"{status.get('total_records', 0):,}",
            )
        
        with col3:
            st.metric(
                "Active Meters",
                status.get('meters_count', 0),
            )
        
        with col4:
            uptime = timedelta(seconds=status.get('uptime_seconds', 0))
            st.metric(
                "System Uptime",
                f"{uptime.days}d {uptime.seconds//3600}h {(uptime.seconds//60)%60}m"
            )
    except Exception as e:
        st.error(f"Error rendering system overview: {str(e)}")

def render_meter_details():
    """Render the meter details section"""
    st.header("Meter Details")
    
    try:
        meters = get_meter_status()
        if not meters:
            st.warning("No meter information available")
            return
        
        for meter in meters:
            with st.expander(f"📊 {meter['meter_name']}", expanded=True):
                col1, col2, col3 = st.columns(3)
                
                with col1:
                    last_power = meter.get('last_power_reading', 0)
                    if isinstance(last_power, str):
                        try:
                            last_power = float(last_power)
                        except ValueError:
                            last_power = 0
                    st.metric(
                        "Current Power",
                        f"{last_power:.2f} W"
                    )
                
                with col2:
                    st.metric(
                        "Total Readings",
                        f"{meter.get('total_readings', 0):,}"
                    )
                
                with col3:
                    # Debug timestamp value
                    raw_timestamp = meter.get('last_reading_timestamp')
                    st.caption(f"Debug - Raw timestamp: {raw_timestamp}")
                    
                    last_reading = format_timestamp(raw_timestamp)
                    if last_reading:
                        try:
                            last_time = datetime.strptime(last_reading, '%Y-%m-%d %H:%M:%S')
                            time_diff = datetime.now() - last_time
                            status_color = "🟢" if time_diff.seconds < 300 else "🔴"
                            st.metric(
                                "Last Update",
                                f"{status_color} {last_reading}"
                            )
                        except Exception as e:
                            st.metric(
                                "Last Update",
                                "🔴 Error parsing time"
                            )
                            st.caption(f"Debug - Error: {str(e)}")
                    else:
                        st.metric(
                            "Last Update",
                            "🔴 No data"
                        )
                
                try:
                    df = load_data(
                        datetime.now() - timedelta(hours=24),
                        datetime.now(),
                        meter['meter_name']
                    )
                    
                    if not df.empty:
                        col1, col2 = st.columns(2)
                        
                        with col1:
                            # Power stability gauge
                            power_std = df['total_power'].std()
                            max_power = df['total_power'].max()
                            stability = max(0, min(100, 100 * (1 - power_std / max_power if max_power != 0 else 1)))
                            
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
                            reliability = min(100, 100 * (expected_interval / avg_interval if avg_interval != 0 else 1))
                            
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
                    else:
                        st.info("No data available for the last 24 hours")
                except Exception as e:
                    st.error(f"Error loading meter data: {str(e)}")
    except Exception as e:
        st.error(f"Error rendering meter details: {str(e)}")

def render_database_stats():
    """Render the database statistics section"""
    st.header("Database Statistics")
    
    try:
        df = load_data(
            datetime.now() - timedelta(days=30),
            datetime.now()
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
    except Exception as e:
        st.error(f"Error rendering database stats: {str(e)}")

def main():
    try:
        render_system_overview()
        st.markdown("---")
        render_meter_details()
        st.markdown("---")
        render_database_stats()
    except Exception as e:
        st.error(f"Main execution error: {str(e)}")

if __name__ == "__main__":
    main()