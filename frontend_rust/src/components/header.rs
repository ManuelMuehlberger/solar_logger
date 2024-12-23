use maud::{html, Markup};
use crate::utils::types::WeatherInfo;

pub fn render_header(weather: Option<WeatherInfo>) -> Markup {
    html! {
        header class="header-container" {
            div class="header-grid" {
                div class="time-section" {
                    h2 class="time" id="current-time" { "Loading..." }
                    p class="date" id="current-date" { "Loading..." }
                }

                div class="title-section" {
                    h1 { "☀️ Solar Dashboard" }
                }

                div class="weather-section" {
                    @if let Some(weather) = weather {
                        h2 { "🌡️ " (format!("{:.1}°C", weather.temperature)) }
                        p { (weather.description) }
                        p class="humidity" { "💧 Humidity: " (weather.humidity) "%" }
                        p class="sun-times" {
                            "🌅 " (weather.sunrise) " | 🌇 " (weather.sunset)
                        }
                    } @else {
                        p { "Weather data unavailable" }
                    }
                }
            }
        }

        // Inline script for updating time
        script {
            (r#"
                function updateDateTime() {
                    const now = new Date();
                    
                    // Update time
                    const timeElement = document.getElementById('current-time');
                    timeElement.textContent = now.toLocaleTimeString();
                    
                    // Update date
                    const dateElement = document.getElementById('current-date');
                    dateElement.textContent = now.toLocaleDateString('en-US', {
                        month: 'long',
                        day: 'numeric',
                        year: 'numeric'
                    });
                }

                // Update immediately and then every second
                updateDateTime();
                setInterval(updateDateTime, 1000);
            "#)
        }

        style {
            (r#"
                .header-container {
                    width: 100%;
                    background: #ffffff;
                    padding: 1rem;
                    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                }

                .header-grid {
                    display: grid;
                    grid-template-columns: 1fr 2fr 1fr;
                    gap: 2rem;
                    align-items: center;
                }

                .time-section {
                    text-align: left;
                }

                .time {
                    font-size: 2rem;
                    margin: 0;
                    color: #2d3748;
                }

                .date {
                    font-size: 1.2rem;
                    margin: 0.5rem 0;
                    color: #4a5568;
                }

                .title-section {
                    text-align: center;
                }

                .title-section h1 {
                    font-size: 2.5rem;
                    margin: 0;
                    color: #2d3748;
                }

                .weather-section {
                    text-align: right;
                }

                .weather-section h2 {
                    font-size: 1.8rem;
                    margin: 0;
                    color: #2d3748;
                }

                .weather-section p {
                    margin: 0.3rem 0;
                    color: #4a5568;
                }

                .humidity, .sun-times {
                    font-size: 0.9rem;
                }
            "#)
        }
    }
}