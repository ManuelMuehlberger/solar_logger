use maud::{html, Markup};

pub fn render_sidebar() -> Markup {
    html! {
        aside class="sidebar" {
            div class="sidebar-content" {
                h2 { "System Status" }

                // Backend Status Section
                section class="status-section" {
                    h3 { "🖥️ Backend Status" }
                    div id="backend-status" class="status-card" {
                        p class="status-indicator" {"Checking connection..."}
                        
                        div class="metrics-container" style="display: none;" {
                            div class="metric" {
                                label { "Database Size" }
                                span id="database-size" { "..." }
                            }
                            
                            div class="metric" {
                                label { "Records" }
                                span id="total-records" { "..." }
                            }
                        }

                        p class="last-update" id="backend-last-update" {
                            "Last Updated: ..."
                        }
                    }
                }

                // Meter Status Section
                section class="meters-section" {
                    h3 { "📊 Power Meters" }
                    div id="sidebar-meters-container" {
                        p { "Loading meter data..." }
                    }
                }

                // System Info Footer
                footer class="sidebar-footer" {
                    small { "System Information" }
                    small id="server-time" { "Server Time: Loading..." }
                }
            }
        }

        style {
            (r#"
                .sidebar {
                    width: 300px;
                    background: #ffffff;
                    height: 100vh;
                    position: fixed;
                    top: 0;
                    left: 0;
                    overflow-y: auto;
                    box-shadow: 2px 0 5px rgba(0, 0, 0, 0.1);
                }

                .sidebar-content {
                    padding: 1.5rem;
                }

                .sidebar h2 {
                    font-size: 1.5rem;
                    color: #2d3748;
                    margin: 0 0 1.5rem 0;
                }

                .sidebar h3 {
                    font-size: 1.2rem;
                    color: #4a5568;
                    margin: 1.5rem 0 1rem 0;
                }

                .status-card {
                    background: #f7fafc;
                    border-radius: 8px;
                    padding: 1rem;
                    margin-bottom: 1rem;
                }

                .status-indicator {
                    margin: 0 0 1rem 0;
                    font-weight: 500;
                }

                .status-indicator.connected {
                    color: #48bb78;
                }

                .status-indicator.disconnected {
                    color: #f56565;
                }

                .metrics-container {
                    display: grid;
                    grid-template-columns: 1fr 1fr;
                    gap: 1rem;
                    margin: 1rem 0;
                }

                .metric {
                    display: flex;
                    flex-direction: column;
                }

                .metric label {
                    font-size: 0.875rem;
                    color: #718096;
                    margin-bottom: 0.25rem;
                }

                .metric span {
                    font-weight: 600;
                    color: #2d3748;
                }

                .last-update {
                    font-size: 0.875rem;
                    color: #718096;
                    margin: 0;
                }

                .sidebar-meters {
                    margin-top: 1rem;
                }

                .meter-card {
                    background: #f7fafc;
                    border-radius: 8px;
                    padding: 1rem;
                    margin-bottom: 0.5rem;
                }

                .meter-card h4 {
                    margin: 0 0 0.5rem 0;
                    color: #2d3748;
                }

                .meter-reading {
                    font-size: 1.25rem;
                    font-weight: 600;
                    color: #2d3748;
                    margin: 0.5rem 0;
                }

                .meter-update-time {
                    font-size: 0.75rem;
                    color: #718096;
                }

                .sidebar-footer {
                    margin-top: 2rem;
                    padding-top: 1rem;
                    border-top: 1px solid #e2e8f0;
                }

                .sidebar-footer small {
                    display: block;
                    color: #718096;
                    margin-bottom: 0.25rem;
                }

                /* Add padding to main content to account for sidebar */
                main.content {
                    margin-left: 300px;
                }
            "#)
        }

        script {
            (r#"
            let lastStatusUpdate = Date.now();

            async function updateBackendStatus() {
                try {
                    const response = await fetch('/api/status');
                    const status = await response.json();
                    
                    const statusCard = document.getElementById('backend-status');
                    const metricsContainer = statusCard.querySelector('.metrics-container');
                    
                    // Update connection status
                    const statusIndicator = statusCard.querySelector('.status-indicator');
                    statusIndicator.textContent = 'Connected';
                    statusIndicator.classList.add('connected');
                    statusIndicator.classList.remove('disconnected');
                    
                    // Show metrics
                    metricsContainer.style.display = 'grid';
                    
                    // Update metrics
                    document.getElementById('database-size').textContent = 
                        `${(status.database_size_bytes / 1024 / 1024).toFixed(1)} MB`;
                    document.getElementById('total-records').textContent = 
                        status.total_records.toLocaleString();
                    
                    // Update timestamp
                    lastStatusUpdate = Date.now();
                    updateLastUpdate();
                    
                } catch (error) {
                    const statusCard = document.getElementById('backend-status');
                    const statusIndicator = statusCard.querySelector('.status-indicator');
                    statusIndicator.textContent = 'Disconnected';
                    statusIndicator.classList.add('disconnected');
                    statusIndicator.classList.remove('connected');
                    
                    const metricsContainer = statusCard.querySelector('.metrics-container');
                    metricsContainer.style.display = 'none';
                }
            }

            function updateLastUpdate() {
                const timeSince = Math.round((Date.now() - lastStatusUpdate) / 1000);
                const lastUpdateElement = document.getElementById('backend-last-update');
                lastUpdateElement.textContent = `Last Updated: ${timeSince} seconds ago`;
            }

            async function updateSidebarMeters() {
                try {
                    const response = await fetch('/api/meters');
                    const meters = await response.json();
                    
                    const container = document.getElementById('sidebar-meters-container');
                    container.innerHTML = meters.map(meter => `
                        <div class="meter-card">
                            <h4>📍 ${meter.meter_name}</h4>
                            <div class="meter-reading">
                                ${meter.last_power_reading.toFixed(2)} W
                            </div>
                            <div class="meter-update-time">
                                Last update: ${new Date(meter.last_reading_timestamp * 1000).toLocaleTimeString()}
                            </div>
                        </div>
                    `).join('');
                } catch (error) {
                    console.error('Error updating sidebar meters:', error);
                }
            }

            function updateServerTime() {
                const timeElement = document.getElementById('server-time');
                timeElement.textContent = `Server Time: ${new Date().toLocaleString()}`;
            }

            // Update backend status every 5 seconds
            setInterval(updateBackendStatus, 5000);
            setInterval(updateLastUpdate, 1000);
            
            // Update meters every 5 seconds
            setInterval(updateSidebarMeters, 5000);
            
            // Update server time every second
            setInterval(updateServerTime, 1000);

            // Initial updates
            updateBackendStatus();
            updateSidebarMeters();
            updateServerTime();
            "#)
        }
    }
}