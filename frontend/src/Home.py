import streamlit as st
from datetime import datetime
import pytz
import dateutil.parser
from utils.database import (
    get_current_power_usage,
    get_daily_stats,
    get_meter_status,
    get_backend_status
)
from utils.weather import get_weather, get_daylight_info
import plotly.graph_objects as go
from components.sidebar import render_sidebar
import time

# Page config
st.set_page_config(
    page_title="Solar Dashboard",
    page_icon="☀️",
    layout="wide"
)

def render_static_header():
    col1, col2, col3 = st.columns([2,3,2])
    with col2:
        st.title("☀️ Solar Dashboard")

def render_time_and_weather(container):
    col1, col2 = container.columns([1,1])
    with col1:
        local_time = datetime.now().strftime("%H:%M:%S")
        local_date = datetime.now().strftime("%B %d, %Y")
        col1.header(f"⏰ {local_time}")
        col1.subheader(local_date)
    
    with col2:
        weather = get_weather()
        if weather:
            col2.header(f"🌡️ {weather['temperature']}°C")
            col2.subheader(f"{weather['description']}")
            col2.write(f"💧 Humidity: {weather['humidity']}%")
            col2.write(f"🌅 Sunrise: {weather['sunrise']} | 🌇 Sunset: {weather['sunset']}")

def render_power_metrics(container):
    col1, col2, col3, col4 = container.columns(4)
    current_power = get_current_power_usage()
    daily_stats = get_daily_stats()
    
    with col1:
        col1.metric(
            "Current Power Usage",
            f"{current_power:.1f} W",
            delta=f"{current_power - daily_stats['average_power']:.1f} W"
        )
    
    with col2:
        col2.metric(
            "Today's Energy Import",
            f"{daily_stats['total_import']:.2f} kWh"
        )
    
    with col3:
        col3.metric(
            "Today's Energy Export",
            f"{daily_stats['total_export']:.2f} kWh"
        )
    
    with col4:
        col4.metric(
            "Peak Power Today",
            f"{daily_stats['peak_power']:.1f} W"
        )

def render_power_gauge(container):
    current_power = get_current_power_usage()
    fig = go.Figure(go.Indicator(
        mode = "gauge+number",
        value = current_power,
        domain = {'x': [0, 1], 'y': [0, 1]},
        title = {'text': "Current Power Usage (W)"},
        gauge = {
            'axis': {'range': [None, max(5000, current_power * 1.2)]},
            'steps': [
                {'range': [0, 2000], 'color': "lightgray"},
                {'range': [2000, 4000], 'color': "gray"}
            ],
            'threshold': {
                'line': {'color': "red", 'width': 4},
                'thickness': 0.75,
                'value': current_power
            }
        }
    ))
    
    container.plotly_chart(fig, use_container_width=True)

def render_meter_status(container):
    meters = get_meter_status()
    if not meters:
        return
    
    container.header("Meter Status")
    cols = container.columns(len(meters))
    
    for idx, meter in enumerate(meters):
        with cols[idx]:
            cols[idx].metric(
                meter['meter_name'],
                f"{meter['last_power_reading']:.1f} W"
            )
            try:
                last_update = dateutil.parser.parse(meter['last_reading_timestamp'])
                cols[idx].write(f"Last Update: {last_update.strftime('%H:%M:%S')}")
            except Exception as e:
                cols[idx].write("⚠️ Invalid timestamp")
                cols[idx].caption(f"Raw timestamp: {meter['last_reading_timestamp']}")

def main():
    render_sidebar()
    render_static_header()
    st.markdown("---")
    
    # Create placeholders for dynamic content
    time_weather_placeholder = st.empty()
    metrics_placeholder = st.empty()
    st.markdown("---")
    
    col1, col2 = st.columns([2,1])
    with col1:
        gauge_placeholder = st.empty()
    with col2:
        meter_placeholder = st.empty()

    # Update the components
    render_time_and_weather(time_weather_placeholder)
    render_power_metrics(metrics_placeholder)
    render_power_gauge(gauge_placeholder)
    render_meter_status(meter_placeholder)
    
    # Add last update time
    st.markdown(
        f"<div style='text-align: center; color: gray;'>Last update: {datetime.now().strftime('%H:%M:%S')}</div>",
        unsafe_allow_html=True
    )

    # Schedule next refresh
    time.sleep(5)
    st.rerun()

if __name__ == "__main__":
    main()