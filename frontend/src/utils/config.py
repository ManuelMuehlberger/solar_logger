# utils/config.py
import os
import tomli
from dataclasses import dataclass
from typing import Dict, Optional
from zoneinfo import ZoneInfo

BACKEND_URL = "http://localhost:8081"
DB_PATH = "/home/manu/solarmeter/db/solar_db.db"

@dataclass
class LocationConfig:
    city: str
    timezone: str
    latitude: float
    longitude: float

@dataclass
class GlobalConfig:
    database_url: str
    create_database: bool = False
    health_check_port: Optional[int] = None
    web_server_port: Optional[int] = 8081
    bind_address: str = "127.0.0.1"

@dataclass
class MeterConfig:
    name: str
    type: str
    port: Optional[str] = None
    baud_rate: Optional[int] = None
    timeout: Optional[int] = None
    polling_rate: Optional[int] = None
    modbus_address: Optional[int] = None

class Config:
    def __init__(self, config_path: str = None):
        # Default location config
        self.location = LocationConfig(
            city="Berlin",
            timezone="Europe/Berlin",
            latitude=52.520008,
            longitude=13.404954
        )

        if config_path is None:
            # Look for config in standard locations
            locations = [
                "./config.toml",
                "../config.toml",
                "/etc/solarmeter/config.toml",
                os.path.expanduser("~/solarmeter/config.toml")
            ]
            for loc in locations:
                if os.path.exists(loc):
                    config_path = loc
                    break

        self.global_config = GlobalConfig(
            database_url=DB_PATH,  # Default to existing config
        )

        if config_path and os.path.exists(config_path):
            try:
                with open(config_path, "rb") as f:
                    config_data = tomli.load(f)
                
                # Parse global config
                global_data = config_data.get("global", {})
                self.global_config = GlobalConfig(
                    database_url=os.path.expanduser(global_data.get("database_url", DB_PATH)),
                    create_database=global_data.get("create_database", False),
                    health_check_port=global_data.get("health_check_port"),
                    web_server_port=global_data.get("web_server_port", 8081),
                    bind_address=global_data.get("bind_address", "127.0.0.1"),
                )

                # Parse location config
                location_data = config_data.get("location", {})
                self.location = LocationConfig(
                    city=location_data.get("city", "Berlin"),
                    timezone=location_data.get("timezone", "Europe/Berlin"),
                    latitude=location_data.get("latitude", 52.520008),
                    longitude=location_data.get("longitude", 13.404954)
                )

                # Parse meters config
                self.meters: Dict[str, MeterConfig] = {}
                for meter_id, meter_data in config_data.get("meters", {}).items():
                    self.meters[meter_id] = MeterConfig(
                        name=meter_data.get("name"),
                        type=meter_data.get("type"),
                        port=meter_data.get("port"),
                        baud_rate=meter_data.get("baud_rate"),
                        timeout=meter_data.get("timeout"),
                        polling_rate=meter_data.get("polling_rate"),
                        modbus_address=meter_data.get("modbus_address")
                    )
            except Exception as e:
                print(f"Error loading config file: {e}, using defaults")

    @property
    def database_path(self) -> str:
        return self.global_config.database_url

    def get_timezone(self) -> ZoneInfo:
        """Get the configured timezone as a ZoneInfo object"""
        try:
            return ZoneInfo(self.location.timezone)
        except Exception as e:
            print(f"Error getting timezone {self.location.timezone}: {e}, falling back to UTC")
            return ZoneInfo("UTC")

# Create singleton instance
_config: Optional[Config] = None

def get_config() -> Config:
    global _config
    if _config is None:
        _config = Config()
    return _config

# For backward compatibility and easy access
def get_timezone() -> ZoneInfo:
    """Get the configured timezone"""
    return get_config().get_timezone()

# For backward compatibility with weather.py
LOCATION = {
    "city": get_config().location.city,
    "lat": get_config().location.latitude,
    "lon": get_config().location.longitude,
}