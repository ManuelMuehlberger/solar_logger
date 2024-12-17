import streamlit as st
import plotly.graph_objects as go
from datetime import datetime, timedelta
import pandas as pd
import numpy as np
from utils.database import load_data
from utils.config import get_timezone
from components.sidebar import render_sidebar


render_sidebar()

@st.cache_data(ttl=120)
def prepare_power_flow_data(df):
    """Prepare and downsample data for power flow chart"""
    # Ensure numeric columns
    numeric_columns = ['import_power', 'export_power', 'total_power']
    df_clean = df.copy()
    
    # Convert to numeric, replacing errors with NaN
    for col in numeric_columns:
        df_clean[col] = pd.to_numeric(df_clean[col], errors='coerce')
    
    # Drop rows with NaN values
    df_clean = df_clean.dropna(subset=numeric_columns)
    
    # Downsample data to 2-minute intervals
    df_resampled = (df_clean.set_index('timestamp')
                   .resample('1min')
                   .agg({
                       'import_power': 'mean',
                       'export_power': 'mean',
                       'total_power': 'mean'
                   })
                   .reset_index())
    
    return df_resampled

def render_time_selector():
    """Render time range selector with optimized state management"""
    # Initialize session state for time range if not exists
    if 'time_range' not in st.session_state:
        st.session_state.time_range = "Last 24 Hours"
    
    col1, col2 = st.columns([3, 1])
    
    with col1:
        time_range = st.select_slider(
            "Select Time Range",
            options=["Last Hour", "Last 24 Hours", "Last 7 Days", "Last 30 Days", "Custom"],
            value=st.session_state.time_range,
            key='time_range_slider'
        )
    
    local_tz = get_timezone()
    now = datetime.now(local_tz)
    
    # Calculate time range based on selection
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
    else:  # Custom range
        with col2:
            start_date = st.date_input(
                "Start Date",
                value=now.date() - timedelta(days=7),
                key='custom_start_date'
            )
            end_date = st.date_input(
                "End Date",
                value=now.date(),
                key='custom_end_date'
            )
            start_time = datetime.combine(start_date, datetime.min.time()).replace(tzinfo=local_tz)
            end_time = datetime.combine(end_date, datetime.max.time()).replace(tzinfo=local_tz)
    
    # Update session state
    st.session_state.time_range = time_range
    
    return start_time, end_time

@st.cache_data(ttl=120)
def prepare_daily_patterns(df):
    """Prepare data for daily patterns chart"""
    df_clean = df.copy()
    
    # Convert to numeric
    df_clean['total_power'] = pd.to_numeric(df_clean['total_power'], errors='coerce')
    df_clean['hour'] = df_clean['timestamp'].dt.hour
    
    # Calculate hourly averages
    hourly_avg = (df_clean.groupby(['meter_name', 'hour'])['total_power']
                 .mean()
                 .reset_index())
    
    return hourly_avg

def render_power_flow(df):
    """Render power flow chart with optimized data"""
    st.header("Power Flow Analysis")
    
    try:
        df_plot = prepare_power_flow_data(df)
        
        fig = go.Figure()
        
        # Efficient trace creation
        traces = [
            go.Scatter(
                x=df_plot['timestamp'],
                y=df_plot['import_power'],
                name='Import Power',
                fill='tozeroy',
                line=dict(color='rgba(239, 85, 59, 0.8)'),
                hovertemplate='%{y:.1f} W<extra></extra>'
            ),
            go.Scatter(
                x=df_plot['timestamp'],
                y=df_plot['export_power'],
                name='Export Power',
                fill='tozeroy',
                line=dict(color='rgba(99, 110, 250, 0.8)'),
                hovertemplate='%{y:.1f} W<extra></extra>'
            )
        ]
        
        fig.add_traces(traces)
        
        # Optimized layout
        fig.update_layout(
            title="Power Flow Over Time",
            xaxis_title="Time",
            yaxis_title="Power (W)",
            hovermode='x unified',
            height=400,
            showlegend=True,
            legend=dict(
                yanchor="top",
                y=0.99,
                xanchor="left",
                x=0.01
            ),
            # Improve rendering performance
            uirevision='constant',
            paper_bgcolor='rgba(0,0,0,0)',
            plot_bgcolor='rgba(0,0,0,0)',
            margin=dict(l=50, r=20, t=40, b=50)
        )
        
        # Optimized config
        config = {
            'displayModeBar': False,
            'responsive': True,
            'staticPlot': False,
            'scrollZoom': False
        }
        
        placeholder = st.empty()
        with placeholder.container():
            st.plotly_chart(fig, use_container_width=True, config=config)
            
    except Exception as e:
        st.error(f"Error rendering power flow chart: {str(e)}")

def render_daily_patterns(df):
    """Render daily patterns chart with optimized data"""
    st.header("Daily Power Patterns")
    
    try:
        hourly_avg = prepare_daily_patterns(df)
        
        fig = go.Figure()
        
        for meter in hourly_avg['meter_name'].unique():
            meter_data = hourly_avg[hourly_avg['meter_name'] == meter]
            fig.add_trace(go.Scatter(
                x=meter_data['hour'],
                y=meter_data['total_power'],
                name=f"{meter}",
                mode='lines+markers'
            ))
        
        fig.update_layout(
            title="Average Power by Hour of Day",
            xaxis_title="Hour",
            yaxis_title="Average Power (W)",
            xaxis=dict(tickmode='linear', tick0=0, dtick=1),
            height=400,
            uirevision='constant'
        )
        
        st.plotly_chart(fig, use_container_width=True)
        
    except Exception as e:
        st.error(f"Error rendering daily patterns chart: {str(e)}")

def main():
    st.title("📊 Solar Power Analytics")
    
    # Use session state to avoid unnecessary reloading
    if 'start_time' not in st.session_state:
        st.session_state.start_time = datetime.now(get_timezone()) - timedelta(days=1)
    if 'end_time' not in st.session_state:
        st.session_state.end_time = datetime.now(get_timezone())
    
    start_time, end_time = render_time_selector()
    
    # Only reload data if time range changed
    if (start_time != st.session_state.start_time or 
        end_time != st.session_state.end_time):
        st.session_state.start_time = start_time
        st.session_state.end_time = end_time
        st.session_state.df = load_data(start_time, end_time)
    
    df = st.session_state.df
    
    if df.empty:
        st.warning("No data available for the selected time range")
        return
    
    # Create placeholder containers
    flow_container = st.empty()
    patterns_container = st.empty()
    
    # Render charts in containers
    with flow_container:
        render_power_flow(df)
    with patterns_container:
        render_daily_patterns(df)

if __name__ == "__main__":
    main()