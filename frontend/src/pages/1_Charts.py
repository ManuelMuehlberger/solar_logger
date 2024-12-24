import streamlit as st
import plotly.express as px
import plotly.graph_objects as go
from datetime import datetime, timedelta
import pandas as pd
from utils.database import load_data
from utils.config import get_timezone
from components.sidebar import render_sidebar

st.set_page_config(page_title="Solar Analytics", page_icon="📊", layout="wide")

render_sidebar()

def render_time_selector():
    col1, col2 = st.columns([3, 1])
    
    with col1:
        time_range = st.select_slider(
            "Select Time Range",
            options=["Last Hour", "Last 24 Hours", "Last 7 Days", "Last 30 Days", "Custom"],
            value="Last 24 Hours"
        )
    
    local_tz = get_timezone()
    now = datetime.now(local_tz)
    
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
        with col2:
            start_date = st.date_input("Start Date", value=now.date() - timedelta(days=7))
            end_date = st.date_input("End Date", value=now.date())
            start_time = datetime.combine(start_date, datetime.min.time()).replace(tzinfo=local_tz)
            end_time = datetime.combine(end_date, datetime.max.time()).replace(tzinfo=local_tz)
    
    return start_time, end_time

def render_power_flow(df):
    st.header("Total Power Flow by Meter")
    
    # Create a line plot for each meter's total power
    fig = go.Figure()
    
    for meter in df['meter_name'].unique():
        meter_data = df[df['meter_name'] == meter]
        fig.add_trace(go.Scatter(
            x=meter_data['timestamp'],
            y=meter_data['total_power'],
            name=f'{meter} - Total Power',
            mode='lines',
            line=dict(width=2)
        ))
    
    fig.update_layout(
        title="Total Power Over Time by Meter",
        xaxis_title="Time",
        yaxis_title="Power (W)",
        hovermode='x unified',
        showlegend=True,
        height=500
    )
    
    st.plotly_chart(fig, use_container_width=True)

def render_import_export_flow(df):
    st.header("Import/Export Power Flow")
    
    # Calculate net power (export - import) for each meter
    df['net_power'] = df['export_power'] - df['import_power']
    
    # Create the figure
    fig = go.Figure()
    
    # Add import power (positive values)
    for meter in df['meter_name'].unique():
        meter_data = df[df['meter_name'] == meter]
        fig.add_trace(go.Scatter(
            x=meter_data['timestamp'],
            y=meter_data['import_power'],
            name=f'{meter} - Import',
            fill='tozeroy',
            line=dict(width=1),
            fillcolor='rgba(239, 85, 59, 0.2)'
        ))
    
    # Add export power (negative values)
    for meter in df['meter_name'].unique():
        meter_data = df[df['meter_name'] == meter]
        fig.add_trace(go.Scatter(
            x=meter_data['timestamp'],
            y=-meter_data['export_power'],  # Negative to show below x-axis
            name=f'{meter} - Export',
            fill='tozeroy',
            line=dict(width=1),
            fillcolor='rgba(99, 110, 250, 0.2)'
        ))
    
    fig.update_layout(
        title="Import (↑) and Export (↓) Power Flow",
        xaxis_title="Time",
        yaxis_title="Power (W)",
        hovermode='x unified',
        showlegend=True,
        height=500
    )
    
    st.plotly_chart(fig, use_container_width=True)

def render_daily_patterns(df):
    st.header("Daily Power Patterns")
    
    df['hour'] = df['timestamp'].dt.hour
    
    # Calculate hourly averages for each meter
    hourly_stats = df.groupby(['meter_name', 'hour']).agg({
        'import_power': 'mean',
        'export_power': 'mean',
        'total_power': 'mean'
    }).reset_index()
    
    fig = go.Figure()
    
    for meter in df['meter_name'].unique():
        meter_data = hourly_stats[hourly_stats['meter_name'] == meter]
        
        # Add total power
        fig.add_trace(go.Scatter(
            x=meter_data['hour'],
            y=meter_data['total_power'],
            name=f'{meter} - Avg Power',
            mode='lines+markers'
        ))
    
    fig.update_layout(
        title="Average Power by Hour of Day",
        xaxis_title="Hour",
        yaxis_title="Average Power (W)",
        xaxis=dict(tickmode='linear', tick0=0, dtick=1),
        height=400,
        showlegend=True
    )
    
    st.plotly_chart(fig, use_container_width=True)

def render_energy_summary(df):
    st.header("Energy Summary")
    
    # Group by date and meter
    df['date'] = df['timestamp'].dt.date
    daily_totals = df.groupby(['date', 'meter_name']).agg({
        'import_power': lambda x: (x * pd.Timedelta('1h') / pd.Timedelta('1s')).sum() / 3600,  # Convert Watt-seconds to Watt-hours
        'export_power': lambda x: (x * pd.Timedelta('1h') / pd.Timedelta('1s')).sum() / 3600
    }).reset_index()
    
    # Calculate net energy
    daily_totals['net_energy_kwh'] = (daily_totals['export_power'] - daily_totals['import_power']) / 1000
    
    # Create bar chart
    fig = go.Figure()
    
    for meter in daily_totals['meter_name'].unique():
        meter_data = daily_totals[daily_totals['meter_name'] == meter]
        fig.add_trace(go.Bar(
            x=meter_data['date'],
            y=meter_data['net_energy_kwh'],
            name=f'{meter}',
            marker_color=meter_data['net_energy_kwh'].apply(
                lambda x: 'rgba(99, 110, 250, 0.8)' if x >= 0 else 'rgba(239, 85, 59, 0.8)'
            )
        ))
    
    fig.update_layout(
        title="Daily Net Energy by Meter (kWh)",
        barmode='group',
        xaxis_title="Date",
        yaxis_title="Net Energy (kWh)",
        height=400,
        showlegend=True
    )
    
    st.plotly_chart(fig, use_container_width=True)

def main():
    st.title("📊 Solar Power Analytics")
    
    start_time, end_time = render_time_selector()
    df = load_data(start_time, end_time)
    
    if df.empty:
        st.warning("No data available for the selected time range")
        return
    
    render_power_flow(df)
    render_import_export_flow(df)
    render_daily_patterns(df)
    render_energy_summary(df)
    
    # Add raw data download option
    st.markdown("---")
    st.header("Raw Data Export")
    csv = df.to_csv(index=False).encode('utf-8')
    st.download_button(
        "Download Data as CSV",
        csv,
        "solar_data.csv",
        "text/csv"
    )

if __name__ == "__main__":
    main()