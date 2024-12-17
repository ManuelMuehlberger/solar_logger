# utils/weather.py
import requests
from datetime import datetime
import pytz
from utils.config import get_config

def get_weather():
    """Fetch current weather data from Open-Meteo API"""
    try:
        config = get_config()
        # Current weather API endpoint
        url = "https://api.open-meteo.com/v1/forecast"
        params = {
            "latitude": config.location.latitude,
            "longitude": config.location.longitude,
            "current": ["temperature_2m", "relative_humidity_2m", "weather_code"],
            "timezone": config.location.timezone
        }
        
        response = requests.get(url, params=params)
        data = response.json()
        
        if response.status_code == 200:
            # Get weather description based on WMO code
            weather_code = data["current"]["weather_code"]
            description = get_weather_description(weather_code)
            
            # Get sun times
            sun_times = get_sun_times()
            
            weather_info = {
                "temperature": round(data["current"]["temperature_2m"], 1),
                "humidity": data["current"]["relative_humidity_2m"],
                "description": description,
                "sunrise": sun_times.get("sunrise", "N/A"),
                "sunset": sun_times.get("sunset", "N/A"),
            }
            return weather_info
    except Exception as e:
        print(f"Error fetching weather: {e}")
    
    return None

def get_sun_times():
    """Fetch sunrise and sunset times"""
    try:
        config = get_config()
        url = "https://api.open-meteo.com/v1/forecast"
        params = {
            "latitude": config.location.latitude,
            "longitude": config.location.longitude,
            "daily": ["sunrise", "sunset"],
            "timezone": config.location.timezone
        }
        
        response = requests.get(url, params=params)
        data = response.json()
        
        if response.status_code == 200:
            # Get today's sunrise and sunset
            sunrise = datetime.fromisoformat(data["daily"]["sunrise"][0]).strftime("%H:%M")
            sunset = datetime.fromisoformat(data["daily"]["sunset"][0]).strftime("%H:%M")
            
            return {
                "sunrise": sunrise,
                "sunset": sunset
            }
    except Exception as e:
        print(f"Error fetching sun times: {e}")
    
    return {}

def get_daylight_info():
    """Calculate daylight hours and solar noon"""
    sun_times = get_sun_times()
    if not sun_times:
        return None
        
    sunrise = datetime.strptime(sun_times["sunrise"], "%H:%M")
    sunset = datetime.strptime(sun_times["sunset"], "%H:%M")
    
    daylight_hours = sunset.hour - sunrise.hour + (sunset.minute - sunrise.minute) / 60
    solar_noon = sunrise + (sunset - sunrise) / 2
    
    return {
        "daylight_hours": round(daylight_hours, 1),
        "solar_noon": solar_noon.strftime("%H:%M")
    }

def get_weather_description(code):
    """Convert WMO Weather Code to description
    Codes from: https://open-meteo.com/en/docs"""
    weather_codes = {
        0: "Clear sky",
        1: "Mainly clear",
        2: "Partly cloudy",
        3: "Overcast",
        45: "Foggy",
        48: "Depositing rime fog",
        51: "Light drizzle",
        53: "Moderate drizzle",
        55: "Dense drizzle",
        61: "Slight rain",
        63: "Moderate rain",
        65: "Heavy rain",
        71: "Slight snow",
        73: "Moderate snow",
        75: "Heavy snow",
        77: "Snow grains",
        80: "Slight rain showers",
        81: "Moderate rain showers",
        82: "Violent rain showers",
        85: "Slight snow showers",
        86: "Heavy snow showers",
        95: "Thunderstorm",
        96: "Thunderstorm with slight hail",
        99: "Thunderstorm with heavy hail",
    }
    return weather_codes.get(code, "Unknown")