import streamlit as st
import plotly.express as px
import plotly.graph_objects as go
from datetime import datetime, timedelta
import pandas as pd
from utils.database import load_data
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
        with col2:
            start_time = st.date_input("Start Date", value=now - timedelta(days=7))
            end_time = st.date_input("End Date", value=now)
            start_time = datetime.combine(start_time, datetime.min.time())
            end_time = datetime.combine(end_time, datetime.max.time())
    
    return start_time, end_time

def render_power_flow(df):
    st.header("Power Flow Analysis")
    
    # Calculate net power flow
    df['net_power'] = df['export_power'] - df['import_power']
    
    fig = go.Figure()
    
    # Add import power
    fig.add_trace(go.Scatter(
        x=df['timestamp'],
        y=df['import_power'],
        name='Import Power',
        fill='tozeroy',
        line=dict(color='rgba(239, 85, 59, 0.8)')
    ))
    
    # Add export power
    fig.add_trace(go.Scatter(
        x=df['timestamp'],
        y=df['export_power'],
        name='Export Power',
        fill='tozeroy',
        line=dict(color='rgba(99, 110, 250, 0.8)')
    ))
    
    fig.update_layout(
        title="Power Flow Over Time",
        xaxis_title="Time",
        yaxis_title="Power (W)",
        hovermode='x unified',
        height=400
    )
    
    st.plotly_chart(fig, use_container_width=True)

def render_daily_patterns(df):
    st.header("Daily Power Patterns")
    
    df['hour'] = df['timestamp'].dt.hour
    hourly_avg = df.groupby(['meter_name', 'hour']).agg({
        'total_power': 'mean',
        'import_power': 'mean',
        'export_power': 'mean'
    }).reset_index()
    
    fig = go.Figure()
    
    for meter in df['meter_name'].unique():
        meter_data = hourly_avg[hourly_avg['meter_name'] == meter]
        fig.add_trace(go.Scatter(
            x=meter_data['hour'],
            y=meter_data['total_power'],
            name=f"{meter} - Total Power",
            mode='lines+markers'
        ))
    
    fig.update_layout(
        title="Average Power by Hour of Day",
        xaxis_title="Hour",
        yaxis_title="Average Power (W)",
        xaxis=dict(tickmode='linear', tick0=0, dtick=1),
        height=400
    )
    
    st.plotly_chart(fig, use_container_width=True)

def render_energy_distribution(df):
    st.header("Energy Distribution")
    
    col1, col2 = st.columns(2)
    
    with col1:
        # Calculate daily totals
        df['date'] = df['timestamp'].dt.date
        daily_totals = df.groupby('date').agg({
            'import_power': 'sum',
            'export_power': 'sum'
        }).reset_index()
        
        daily_totals['net_energy'] = (daily_totals['export_power'] - daily_totals['import_power']) / 3600  # Convert to kWh
        
        fig = go.Figure()
        fig.add_trace(go.Bar(
            x=daily_totals['date'],
            y=daily_totals['net_energy'],
            name='Net Energy',
            marker_color=daily_totals['net_energy'].apply(
                lambda x: 'rgba(99, 110, 250, 0.8)' if x >= 0 else 'rgba(239, 85, 59, 0.8)'
            )
        ))
        
        fig.update_layout(
            title="Daily Net Energy (kWh)",
            height=400
        )
        
        st.plotly_chart(fig, use_container_width=True)
    
    with col2:
        # Create energy balance pie chart
        total_import = df['import_power'].sum() / 3600  # Convert to kWh
        total_export = df['export_power'].sum() / 3600  # Convert to kWh
        
        fig = go.Figure(data=[go.Pie(
            labels=['Energy Imported', 'Energy Exported'],
            values=[total_import, total_export],
            hole=.3
        )])
        
        fig.update_layout(
            title="Energy Balance Distribution",
            height=400
        )
        
        st.plotly_chart(fig, use_container_width=True)

def render_comparative_analysis(df):
    st.header("Comparative Analysis")
    
    # Get unique meters for selection
    meters = df['meter_name'].unique()
    if len(meters) > 1:
        selected_meters = st.multiselect("Select Meters to Compare", meters, default=meters)
        df_filtered = df[df['meter_name'].isin(selected_meters)]
    else:
        df_filtered = df
    
    # Create comparative box plot
    fig = go.Figure()
    
    for meter in df_filtered['meter_name'].unique():
        meter_data = df_filtered[df_filtered['meter_name'] == meter]
        
        fig.add_trace(go.Box(
            y=meter_data['total_power'],
            name=meter,
            boxpoints='outliers'
        ))
    
    fig.update_layout(
        title="Power Distribution by Meter",
        yaxis_title="Power (W)",
        height=400
    )
    
    st.plotly_chart(fig, use_container_width=True)

def main():
    st.title("📊 Solar Power Analytics")
    
    start_time, end_time = render_time_selector()
    df = load_data(start_time, end_time)
    
    if df.empty:
        st.warning("No data available for the selected time range")
        return
    
    # Display analytics
    render_power_flow(df)
    render_daily_patterns(df)
    render_energy_distribution(df)
    render_comparative_analysis(df)
    
    # Add data download option
    st.markdown("---")
    st.header("Raw Data Export")
    
    csv = df.to_csv(index=False).encode('utf-8')
    st.download_button(
        "Download Data as CSV",
        csv,
        "solar_data.csv",
        "text/csv",
        key='download-csv'
    )

if __name__ == "__main__":
    main()