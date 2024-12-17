import os
from datetime import datetime, timezone

# Backend settings
BACKEND_URL = "http://localhost:8081"
DB_PATH = "/home/manu/solarmeter/db/solar_db.db"

# Location settings
LOCATION = {
    "city": "Berlin",
    "lat": 48.1374,  # Replace with your latitude
    "lon": 11.5755,  # Replace with your longitude
}

# Time settings
UPDATE_INTERVAL = 120  # seconds

# Theme and styling
THEME = {
    "primary": "#1E88E5",
    "success": "#4CAF50",
    "warning": "#FFC107",
    "danger": "#E53935",
    "info": "#2196F3",
}
