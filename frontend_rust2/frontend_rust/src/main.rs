use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[derive(Default, Clone, Debug, PartialEq)]
struct MeterData {
    meter_name: String,
    timestamp: String,
    total_power: String,
    import_power: String,
    export_power: String,
    total_kwh: String,
}

#[component]
fn App() -> Element {
    let meter_data = Signal::global(|| vec![
        MeterData {
            meter_name: "Meter 1".to_string(),
            timestamp: "2024-08-03 10:00:00".to_string(),
            total_power: "1.2kW".to_string(),
            import_power: "0.8kW".to_string(),
            export_power: "0.4kW".to_string(),
            total_kwh: "150 kWh".to_string(),
        },
        MeterData {
            meter_name: "Meter 2".to_string(),
            timestamp: "2024-08-03 10:15:00".to_string(),
            total_power: "1.5kW".to_string(),
            import_power: "0.9kW".to_string(),
            export_power: "0.6kW".to_string(),
            total_kwh: "152 kWh".to_string(),
        },
    ]);

    rsx! {
        div { class: "container mx-auto mt-10",
            div { class: "bg-white shadow rounded-lg p-4",
                h2 { class: "text-xl font-semibold mb-2", "Meter Data" }

                if meter_data.is_empty() {
                    p { class: "text-gray-500", "No data available" }
                } else {
                    table { class: "min-w-full",
                        thead {
                            tr { class: "bg-gray-100",
                                th { class: "p-2 text-left", "Meter Name" }
                                th { class: "p-2 text-left", "Timestamp" }
                                th { class: "p-2 text-left", "Total Power" }
                                th { class: "p-2 text-left", "Import Power" }
                                th { class: "p-2 text-left", "Export Power" }
                                th { class: "p-2 text-left", "Total kWh" }
                            }
                        }
                        tbody {
                            for data in meter_data.get(2) {
                                tr {
                                    td { class: "p-2", "{data.meter_name}" }
                                    td { class: "p-2", "{data.timestamp}" }
                                    td { class: "p-2", "{data.total_power}" }
                                    td { class: "p-2", "{data.import_power}" }
                                    td { class: "p-2", "{data.export_power}" }
                                    td { class: "p-2", "{data.total_kwh}" }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "bg-white shadow rounded-lg p-4 mt-4",
                h2 { class: "text-xl font-semibold mb-2", "Weather Forecast" }
                p { class: "text-gray-500", "Weather forecast data will be displayed here." }
            }
             div { class: "bg-white shadow rounded-lg p-4 mt-4",
                h2 { class: "text-xl font-semibold mb-2", "Time" }
                p { class: "text-gray-500", "Real-time clock will be displayed here." }
            }
        }
    }
}