# utils/toml_config.py
import tomli
import os
from dataclasses import dataclass
from typing import Dict, Optional

@dataclass
class GlobalConfig:
    database_url: str
    create_database: bool = False
    health_check_port: Optional[int] = None
    web_server_port: Optional[int] = 8081
    bind_address: str = "127.0.0.1"
    timezone: str = "UTC"  # Default to UTC if not specified

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
            if config_path is None:
                raise FileNotFoundError("No config.toml found in standard locations")

        with open(config_path, "rb") as f:
            config_data = tomli.load(f)

        # Parse global config
        global_data = config_data.get("global", {})
        self.global_config = GlobalConfig(
            database_url=os.path.expanduser(global_data.get("database_url", "")),
            create_database=global_data.get("create_database", False),
            health_check_port=global_data.get("health_check_port"),
            web_server_port=global_data.get("web_server_port"),
            bind_address=global_data.get("bind_address", "127.0.0.1"),
            timezone=global_data.get("timezone", "UTC")
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

    @property
    def database_path(self) -> str:
        return self.global_config.database_url

# Create singleton instance
_config: Optional[Config] = None

def get_config() -> Config:
    global _config
    if _config is None:
        _config = Config()
    return _config