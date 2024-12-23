use axum::{
    response::Html,
    routing::get,
    Router,
};
use maud::html;
use crate::components::header;

async fn index() -> Html<String> {
    let markup = html! {
        html {
            head {
                title { "Solar Dashboard" }
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
            }
            body {
                // Render header with initial null weather data
                (header::render_header(None))

                main class="content" {
                    div class="metrics-grid" {
                        div class="metric-card" {
                            h3 { "Current Power Usage" }
                            p id="current-power" { "Loading..." }
                        }

                        div class="metric-card" {
                            h3 { "Today's Energy Import" }
                            p id="energy-import" { "Loading..." }
                        }

                        div class="metric-card" {
                            h3 { "Today's Energy Export" }
                            p id="energy-export" { "Loading..." }
                        }

                        div class="metric-card" {
                            h3 { "Peak Power Today" }
                            p id="peak-power" { "Loading..." }
                        }
                    }

                    div class="meters-section" {
                        h2 { "Meter Status" }
                        div id="meters-container" class="meters-grid" {
                            "Loading meter data..."
                        }
                    }
                }

                style {
                    (r#"
                        body {
                            margin: 0;
                            padding: 0;
                            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen-Sans, Ubuntu, Cantarell, sans-serif;
                            background: #f7fafc;
                        }

                        .content {
                            padding: 2rem;
                            max-width: 1200px;
                            margin: 0 auto;
                        }

                        .metrics-grid {
                            display: grid;
                            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
                            gap: 1.5rem;
                            margin-bottom: 2rem;
                        }

                        .metric-card {
                            background: white;
                            padding: 1.5rem;
                            border-radius: 8px;
                            box-shadow: 0 1px 3px rgba(0,0,0,0.1);
                        }

                        .metric-card h3 {
                            margin: 0 0 1rem 0;
                            color: #4a5568;
                            font-size: 1rem;
                        }

                        .metric-card p {
                            margin: 0;
                            font-size: 1.5rem;
                            font-weight: bold;
                            color: #2d3748;
                        }

                        .meters-section {
                            background: white;
                            padding: 1.5rem;
                            border-radius: 8px;
                            box-shadow: 0 1px 3px rgba(0,0,0,0.1);
                        }

                        .meters-section h2 {
                            margin: 0 0 1.5rem 0;
                            color: #2d3748;
                        }

                        .meters-grid {
                            display: grid;
                            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                            gap: 1rem;
                        }

                        .error-card {
                            background: #fff5f5;
                            border: 1px solid #feb2b2;
                            padding: 1.5rem;
                            border-radius: 8px;
                            text-align: center;
                            color: #c53030;
                        }

                        .error-card p {
                            margin: 0 0 0.5rem 0;
                            font-weight: bold;
                        }

                        .error-card small {
                            color: #742a2a;
                        }
                    "#)
                }

                script type="text/javascript" {
                    (r#"
                    async function updateMeters() {
                        try {
                            const response = await fetch('/api/meters');
                            if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);
                            const meters = await response.json();
                            
                            const container = document.getElementById('meters-container');
                            container.innerHTML = meters.map(meter => {
                                const powerReading = typeof meter.last_power_reading === 'number' 
                                    ? meter.last_power_reading.toFixed(1) 
                                    : 'N/A';
                                    
                                const timestamp = meter.last_reading_timestamp 
                                    ? new Date(meter.last_reading_timestamp * 1000).toLocaleTimeString()
                                    : 'No data';

                                return `
                                    <div class="metric-card">
                                        <h3>${meter.meter_name}</h3>
                                        <p>${powerReading} W</p>
                                        <small>Last update: ${timestamp}</small>
                                    </div>
                                `;
                            }).join('');
                            
                            // Log successful data for debugging
                            console.log('Received meter data:', meters);
                        } catch (error) {
                            console.error('Error fetching meters:', error);
                            const container = document.getElementById('meters-container');
                            container.innerHTML = `
                                <div class="error-card">
                                    <p>Error loading meter data</p>
                                    <small>${error.message}</small>
                                </div>
                            `;
                        }
                    }

                    async function updateWeather() {
                        try {
                            const response = await fetch('/api/weather');
                            if (!response.ok) throw new Error('Weather fetch failed');
                            const weather = await response.json();
                            
                            const weatherSection = document.querySelector('.weather-section');
                            if (weatherSection) {
                                weatherSection.innerHTML = `
                                    <h2>🌡️ ${weather.temperature.toFixed(1)}°C</h2>
                                    <p>${weather.description}</p>
                                    <p class="humidity">💧 Humidity: ${weather.humidity}%</p>
                                    <p class="sun-times">🌅 ${weather.sunrise} | 🌇 ${weather.sunset}</p>
                                `;
                            }
                        } catch (error) {
                            console.error('Error fetching weather:', error);
                        }
                    }

                    // Update every 5 seconds
                    setInterval(updateMeters, 5000);
                    setInterval(updateWeather, 300000); // Weather every 5 minutes

                    // Initial updates
                    updateMeters();
                    updateWeather();
                    "#)
                }
            }
        }
    };

    Html(markup.into_string())
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(index))
}