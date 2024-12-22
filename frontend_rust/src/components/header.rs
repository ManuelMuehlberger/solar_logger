use dioxus::prelude::*;
use crate::utils::types::WeatherInfo;
use chrono::{Local, Timelike};

#[derive(Props, PartialEq)]
pub struct HeaderProps {
    #[props(optional)]
    weather: Option<WeatherInfo>,
}

pub fn Header(cx: Scope<HeaderProps>) -> Element {
    let time = use_state(cx, || Local::now());
    
    // Update time every second
    use_effect(cx, (), |_| async move {
        let mut interval = gloo_timers::future::IntervalStream::new(1000);
        while let Some(_) = interval.next().await {
            time.set(Local::now());
        }
    });

    cx.render(rsx! {
        header {
            class: "header",
            
            // Time section
            div {
                class: "header-time",
                h2 {
                    "{time.format(\"%H:%M:%S\")}"
                }
                p {
                    "{time.format(\"%B %d, %Y\")}"
                }
            }

            // Title section
            div {
                class: "header-title",
                h1 {
                    "☀️ Solar Dashboard"
                }
            }

            // Weather section
            div {
                class: "header-weather",
                if let Some(weather) = &cx.props.weather {
                    rsx! {
                        h2 {
                            "🌡️ {weather.temperature}°C"
                        }
                        p {
                            "{weather.description}"
                        }
                        p {
                            "💧 Humidity: {weather.humidity}%"
                        }
                        p {
                            "🌅 Sunrise: {weather.sunrise} | 🌇 Sunset: {weather.sunset}"
                        }
                    }
                } else {
                    rsx! {
                        p {
                            "Weather data unavailable"
                        }
                    }
                }
            }
        }
    })
}