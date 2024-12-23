# src/components/sidebar.py

import streamlit as st
from datetime import datetime
import dateutil.parser
from utils.database import get_backend_status, get_meter_status

def render_sidebar():
    """Render the persistent sidebar with backend and meter status"""
    with st.sidebar:
        st.title("System Status")
        
        # Backend Status Section
        st.header("🖥️ Backend Status")
        status = get_backend_status()
        
        if status:
            st.success("Connected")
            
            # Backend metrics
            metrics_cols = st.columns(2)
            with metrics_cols[0]:
                st.metric(
                    "Database Size", 
                    f"{status['database_size_bytes'] / 1024 / 1024:.1f} MB"
                )
            with metrics_cols[1]:
                st.metric(
                    "Records", 
                    f"{status['total_records']:,}"
                )
            
            # Last update time
            st.caption(f"Last Updated: {datetime.now().strftime('%H:%M:%S')}")
        else:
            st.error("Disconnected")
            if st.button("🔄 Retry Connection"):
                st.rerun()
        
        # Meter Status Section
        st.markdown("---")
        st.header("📊 Power Meters")
        
        meters = get_meter_status()
        if meters:
            # Create a container for each meter
            for meter in meters:
                with st.expander(f"📍 {meter['meter_name']}", expanded=True):
                    # Current power reading
                    st.metric(
                        "Current Power",
                        f"{meter['last_power_reading']:.2f} W",
                        delta=None  # Could add power change if available
                    )
        else:
            st.warning("No meters connected")
            if st.button("🔄 Refresh Meters"):
                st.rerun()
        
        # System Info Footer
        st.markdown("---")
        st.caption("System Information")
        st.caption(f"Server Time: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")