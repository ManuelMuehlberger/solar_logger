import streamlit as st
from datetime import datetime
import pytz
from utils.database import get_current_power_usage, get_daily_stats
from utils.weather import get_weather, get_daylight_info
import plotly.graph_objects as go
from components.sidebar import render_sidebar

# Page config
st.set_page_config(
    page_title="Solar Dashboard",
    page_icon="☀️",
    layout="wide"
)

# Render the persistent sidebar
render_sidebar()

# Rest of your existing Home.py code...
def render_header():
    col1, col2, col3 = st.columns([2,3,2])
    
    with col1:
        local_time = datetime.now().strftime("%H:%M:%S")
        local_date = datetime.now().strftime("%B %d, %Y")
        st.header(f"⏰ {local_time}")
        st.subheader(local_date)
    
    with col2:
        st.title("☀️ Solar Dashboard")
    
    with col3:
        weather = get_weather()
        if weather:
            st.header(f"🌡️ {weather['temperature']}°C")
            st.subheader(f"{weather['description']}")
            st.write(f"💧 Humidity: {weather['humidity']}%")
            st.write(f"🌅 Sunrise: {weather['sunrise']} | 🌇 Sunset: {weather['sunset']}")

def render_power_metrics():
    col1, col2, col3, col4 = st.columns(4)
    
    current_power = get_current_power_usage()
    daily_stats = get_daily_stats()
    
    with col1:
        st.metric(
            "Current Power Usage",
            f"{current_power:.1f} W",
            delta=f"{current_power - daily_stats['average_power']:.1f} W"
        )
    
    with col2:
        st.metric(
            "Today's Energy Import",
            f"{daily_stats['total_import']:.2f} kWh"
        )
    
    with col3:
        st.metric(
            "Today's Energy Export",
            f"{daily_stats['total_export']:.2f} kWh"
        )
    
    with col4:
        st.metric(
            "Peak Power Today",
            f"{daily_stats['peak_power']:.1f} W"
        )

def render_power_gauge():
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
    
    st.plotly_chart(fig, use_container_width=True)

def render_meter_status():
    meters = get_meter_status()
    if not meters:
        return
    
    st.header("Meter Status")
    cols = st.columns(len(meters))
    
    for idx, meter in enumerate(meters):
        with cols[idx]:
            st.metric(
                meter['meter_name'],
                f"{meter['last_power_reading']:.1f} W"
            )
            last_update = datetime.fromisoformat(meter['last_reading_timestamp'].replace('Z', '+00:00'))
            st.write(f"Last Update: {last_update.strftime('%H:%M:%S')}")

def main():
    render_header()
    st.markdown("---")
    
    render_power_metrics()
    st.markdown("---")
    
    col1, col2 = st.columns([2,1])
    with col1:
        render_power_gauge()
    with col2:
        render_meter_status()

if __name__ == "__main__":
    main()