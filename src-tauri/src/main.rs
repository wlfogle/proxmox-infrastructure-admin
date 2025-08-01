use serde::{Deserialize, Serialize};
use std::process::Command;
use chrono::{DateTime, Utc};
#[derive(Debug, Serialize, Deserialize, Clone)]
struct ContainerInfo {
    id: u32,
    name: String,
    status: String,
    uptime: String,
    cpu_usage: f64,
    memory_usage: f64,
    category: String,
    description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct VMInfo {
    id: u32,
    name: String,
    status: String,
    uptime: String,
    cpu_usage: f64,
    memory_usage: f64,
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SystemOverview {
    containers: Vec<ContainerInfo>,
    vms: Vec<VMInfo>,
    total_containers: u32,
    running_containers: u32,
    total_vms: u32,
    running_vms: u32,
    last_updated: DateTime<Utc>,
}

// System maintenance structures
#[derive(Debug, Serialize, Deserialize, Clone)]
struct ServiceInfo {
    name: String,
    status: String,
    enabled: bool,
    active: bool,
    description: String,
    container_id: Option<u32>,
    vm_id: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BinaryInfo {
    name: String,
    path: String,
    version: String,
    exists: bool,
    executable: bool,
    container_id: Option<u32>,
    vm_id: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ConfigInfo {
    name: String,
    path: String,
    exists: bool,
    readable: bool,
    writable: bool,
    size: u64,
    modified: String,
    container_id: Option<u32>,
    vm_id: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MaintenanceOverview {
    services: Vec<ServiceInfo>,
    binaries: Vec<BinaryInfo>,
    configs: Vec<ConfigInfo>,
    system_health: SystemHealth,
    last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SystemHealth {
    disk_usage: f64,
    memory_usage: f64,
    cpu_load: f64,
    network_status: String,
    uptime: String,
}

// Tauri command to get container status
#[tauri::command]
async fn get_container_status(container_id: u32) -> Result<ContainerInfo, String> {
    let output = Command::new("ssh")
        .args(["proxmox", "pct", "status", &container_id.to_string()])
        .output()
        .map_err(|e| format!("Failed to execute SSH command: {}", e))?;

    if !output.status.success() {
        return Err(format!("Command failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let status_output = String::from_utf8_lossy(&output.stdout);
    let status = if status_output.contains("running") {
        "Running".to_string()
    } else if status_output.contains("stopped") {
        "Stopped".to_string()
    } else {
        "Unknown".to_string()
    };

    Ok(ContainerInfo {
        id: container_id,
        name: format!("Container {}", container_id),
        status,
        uptime: "Unknown".to_string(),
        cpu_usage: 0.0,
        memory_usage: 0.0,
        category: get_container_category(container_id),
        description: get_container_description(container_id),
    })
}

// Tauri command to get VM status
#[tauri::command]
async fn get_vm_status(vm_id: u32) -> Result<VMInfo, String> {
    let output = Command::new("ssh")
        .args(["proxmox", "qm", "status", &vm_id.to_string()])
        .output()
        .map_err(|e| format!("Failed to execute SSH command: {}", e))?;

    if !output.status.success() {
        return Err(format!("Command failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let status_output = String::from_utf8_lossy(&output.stdout);
    let status = if status_output.contains("running") {
        "Running".to_string()
    } else if status_output.contains("stopped") {
        "Stopped".to_string()
    } else {
        "Unknown".to_string()
    };

    Ok(VMInfo {
        id: vm_id,
        name: get_vm_name(vm_id),
        status,
        uptime: "Unknown".to_string(),
        cpu_usage: 0.0,
        memory_usage: 0.0,
        description: get_vm_description(vm_id),
    })
}

// Tauri command to start container
#[tauri::command]
async fn start_container(container_id: u32) -> Result<String, String> {
    let output = Command::new("ssh")
        .args(["proxmox", "pct", "start", &container_id.to_string()])
        .output()
        .map_err(|e| format!("Failed to execute SSH command: {}", e))?;

    if output.status.success() {
        Ok(format!("Container {} started successfully", container_id))
    } else {
        Err(format!("Failed to start container {}: {}", container_id, String::from_utf8_lossy(&output.stderr)))
    }
}

// Tauri command to stop container
#[tauri::command]
async fn stop_container(container_id: u32) -> Result<String, String> {
    let output = Command::new("ssh")
        .args(["proxmox", "pct", "stop", &container_id.to_string()])
        .output()
        .map_err(|e| format!("Failed to execute SSH command: {}", e))?;

    if output.status.success() {
        Ok(format!("Container {} stopped successfully", container_id))
    } else {
        Err(format!("Failed to stop container {}: {}", container_id, String::from_utf8_lossy(&output.stderr)))
    }
}

// Tauri command to restart container
#[tauri::command]
async fn restart_container(container_id: u32) -> Result<String, String> {
    let output = Command::new("ssh")
        .args(["proxmox", "pct", "restart", &container_id.to_string()])
        .output()
        .map_err(|e| format!("Failed to execute SSH command: {}", e))?;

    if output.status.success() {
        Ok(format!("Container {} restarted successfully", container_id))
    } else {
        Err(format!("Failed to restart container {}: {}", container_id, String::from_utf8_lossy(&output.stderr)))
    }
}

// Tauri command to start VM
#[tauri::command]
async fn start_vm(vm_id: u32) -> Result<String, String> {
    let output = Command::new("ssh")
        .args(["proxmox", "qm", "start", &vm_id.to_string()])
        .output()
        .map_err(|e| format!("Failed to execute SSH command: {}", e))?;

    if output.status.success() {
        Ok(format!("VM {} started successfully", vm_id))
    } else {
        Err(format!("Failed to start VM {}: {}", vm_id, String::from_utf8_lossy(&output.stderr)))
    }
}

// Tauri command to stop VM
#[tauri::command]
async fn stop_vm(vm_id: u32) -> Result<String, String> {
    let output = Command::new("ssh")
        .args(["proxmox", "qm", "stop", &vm_id.to_string()])
        .output()
        .map_err(|e| format!("Failed to execute SSH command: {}", e))?;

    if output.status.success() {
        Ok(format!("VM {} stopped successfully", vm_id))
    } else {
        Err(format!("Failed to stop VM {}: {}", vm_id, String::from_utf8_lossy(&output.stderr)))
    }
}

// Helper function to get list of existing containers from Proxmox
async fn get_existing_containers() -> Result<Vec<u32>, String> {
    let output = Command::new("ssh")
        .args(["proxmox", "pct", "list"])
        .output()
        .map_err(|e| format!("Failed to execute SSH command: {}", e))?;

    if !output.status.success() {
        return Err(format!("Command failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut container_ids = Vec::new();
    
    for line in output_str.lines().skip(1) { // Skip header line
        if let Some(id_str) = line.split_whitespace().next() {
            if let Ok(id) = id_str.parse::<u32>() {
                container_ids.push(id);
            }
        }
    }
    
    Ok(container_ids)
}

// Helper function to get list of existing VMs from Proxmox
async fn get_existing_vms() -> Result<Vec<u32>, String> {
    let output = Command::new("ssh")
        .args(["proxmox", "qm", "list"])
        .output()
        .map_err(|e| format!("Failed to execute SSH command: {}", e))?;

    if !output.status.success() {
        return Err(format!("Command failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut vm_ids = Vec::new();
    
    for line in output_str.lines().skip(1) { // Skip header line
        if let Some(id_str) = line.split_whitespace().next() {
            if let Ok(id) = id_str.parse::<u32>() {
                vm_ids.push(id);
            }
        }
    }
    
    Ok(vm_ids)
}

// System maintenance commands

// Tauri command to get maintenance overview
#[tauri::command]
async fn get_maintenance_overview() -> Result<MaintenanceOverview, String> {
    let services = get_all_services().await.unwrap_or_default();
    let binaries = get_all_binaries().await.unwrap_or_default();
    let configs = get_all_configs().await.unwrap_or_default();
    let system_health = get_system_health().await.unwrap_or_default();
    
    Ok(MaintenanceOverview {
        services,
        binaries,
        configs,
        system_health,
        last_updated: Utc::now(),
    })
}

// Tauri command to check service status
#[tauri::command]
async fn check_service_status(service_name: String, container_id: Option<u32>, vm_id: Option<u32>) -> Result<ServiceInfo, String> {
    let target = get_ssh_target(container_id, vm_id);
    
    let output = Command::new("ssh")
        .args([&target, "systemctl", "status", &service_name])
        .output()
        .map_err(|e| format!("Failed to execute SSH command: {}", e))?;
    
    let status_output = String::from_utf8_lossy(&output.stdout);
    let active = status_output.contains("Active: active");
    let enabled_output = Command::new("ssh")
        .args([&target, "systemctl", "is-enabled", &service_name])
        .output()
        .unwrap_or_else(|_| std::process::Output {
            status: std::process::ExitStatus::from_raw(1),
            stdout: Vec::new(),
            stderr: Vec::new(),
        });
    let enabled = String::from_utf8_lossy(&enabled_output.stdout).trim() == "enabled";
    
    Ok(ServiceInfo {
        name: service_name.clone(),
        status: if active { "Active".to_string() } else { "Inactive".to_string() },
        enabled,
        active,
        description: format!("Service: {}", service_name),
        container_id,
        vm_id,
    })
}

// Tauri command to start/stop/restart service
#[tauri::command]
async fn control_service(service_name: String, action: String, container_id: Option<u32>, vm_id: Option<u32>) -> Result<String, String> {
    let target = get_ssh_target(container_id, vm_id);
    
    let output = Command::new("ssh")
        .args([&target, "systemctl", &action, &service_name])
        .output()
        .map_err(|e| format!("Failed to execute SSH command: {}", e))?;
    
    if output.status.success() {
        Ok(format!("Service {} {} successfully", service_name, action))
    } else {
        Err(format!("Failed to {} service {}: {}", action, service_name, String::from_utf8_lossy(&output.stderr)))
    }
}

// Tauri command to check binary
#[tauri::command]
async fn check_binary(binary_name: String, container_id: Option<u32>, vm_id: Option<u32>) -> Result<BinaryInfo, String> {
    let target = get_ssh_target(container_id, vm_id);
    
    // First try 'which' command (searches $PATH)
    let which_output = Command::new("ssh")
        .args([&target, "which", &binary_name])
        .output()
        .map_err(|e| format!("Failed to execute SSH command: {}", e))?;
    
    let mut exists = which_output.status.success();
    let mut path = if exists {
        String::from_utf8_lossy(&which_output.stdout).trim().to_string()
    } else {
        "Not found".to_string()
    };
    
    // If not found in PATH, search common additional locations
    if !exists {
        let search_paths = vec![
            "/opt/*/bin",
            "/opt/bin",
            "/usr/local/sbin",
            "/usr/games",
            "/snap/bin",
            "~/.local/bin",
            "/home/*/bin",
            "/var/lib/*/bin",
            "/srv/*/bin",
            "/app/bin",
            "/config/bin",
            "/data/bin",
            "/media/*/bin",
            "/mnt/*/bin"
        ];
        
        for search_path in &search_paths {
            let find_output = Command::new("ssh")
                .args([&target, "find", search_path, "-name", &binary_name, "-type", "f", "-executable", "2>/dev/null", "|", "head", "-1"])
                .output();
                
            if let Ok(output) = find_output {
                if output.status.success() {
                    let found_path = String::from_utf8_lossy(&output.stdout).trim();
                    if !found_path.is_empty() {
                        exists = true;
                        path = found_path.to_string();
                        break;
                    }
                }
            }
        }
    }
    
    // Get version if binary exists
    let version = if exists {
        let version_output = Command::new("ssh")
            .args([&target, &path, "--version"])
            .output();
            
        if let Ok(output) = version_output {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or("Unknown").to_string()
            } else {
                // Try alternative version commands
                let alt_commands = vec!["-v", "-V", "version", "--help"];
                let mut version_found = "Unknown".to_string();
                
                for cmd in &alt_commands {
                    let alt_output = Command::new("ssh")
                        .args([&target, &path, cmd])
                        .output();
                        
                    if let Ok(alt_out) = alt_output {
                        if alt_out.status.success() {
                            let output_text = String::from_utf8_lossy(&alt_out.stdout);
                            if let Some(first_line) = output_text.lines().next() {
                                if !first_line.is_empty() && first_line.len() < 200 { // Reasonable version string
                                    version_found = first_line.to_string();
                                    break;
                                }
                            }
                        }
                    }
                }
                version_found
            }
        } else {
            "Unknown".to_string()
        }
    } else {
        "N/A".to_string()
    };
    
    // Check if executable
    let executable = if exists {
        let test_output = Command::new("ssh")
            .args([&target, "test", "-x", &path])
            .output();
            
        if let Ok(output) = test_output {
            output.status.success()
        } else {
            false
        }
    } else {
        false
    };
    
    Ok(BinaryInfo {
        name: binary_name,
        path,
        version,
        exists,
        executable,
        container_id,
        vm_id,
    })
}

// Tauri command to check config file
#[tauri::command]
async fn check_config(config_path: String, container_id: Option<u32>, vm_id: Option<u32>) -> Result<ConfigInfo, String> {
    let target = get_ssh_target(container_id, vm_id);
    
    // Check if file exists
    let exists_output = Command::new("ssh")
        .args([&target, "test", "-f", &config_path])
        .output()
        .map_err(|e| format!("Failed to execute SSH command: {}", e))?;
    
    let exists = exists_output.status.success();
    
    let (readable, writable, size, modified) = if exists {
        // Check readable
        let readable_output = Command::new("ssh")
            .args([&target, "test", "-r", &config_path])
            .output()
            .unwrap_or_default();
        let readable = readable_output.status.success();
        
        // Check writable
        let writable_output = Command::new("ssh")
            .args([&target, "test", "-w", &config_path])
            .output()
            .unwrap_or_default();
        let writable = writable_output.status.success();
        
        // Get file stats
        let stat_output = Command::new("ssh")
            .args([&target, "stat", "-c", "%s %Y", &config_path])
            .output()
            .unwrap_or_default();
        
        let stat_str = String::from_utf8_lossy(&stat_output.stdout);
        let parts: Vec<&str> = stat_str.trim().split_whitespace().collect();
        
        let size = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
        let modified = parts.get(1)
            .and_then(|t| t.parse::<i64>().ok())
            .map(|timestamp| {
                DateTime::from_timestamp(timestamp, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or("Unknown".to_string())
            })
            .unwrap_or("Unknown".to_string());
        
        (readable, writable, size, modified)
    } else {
        (false, false, 0, "N/A".to_string())
    };
    
    Ok(ConfigInfo {
        name: config_path.split('/').last().unwrap_or("Unknown").to_string(),
        path: config_path,
        exists,
        readable,
        writable,
        size,
        modified,
        container_id,
        vm_id,
    })
}

// Tauri command to read config file
#[tauri::command]
async fn read_config(config_path: String, container_id: Option<u32>, vm_id: Option<u32>) -> Result<String, String> {
    let target = get_ssh_target(container_id, vm_id);
    
    let output = Command::new("ssh")
        .args([&target, "cat", &config_path])
        .output()
        .map_err(|e| format!("Failed to execute SSH command: {}", e))?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(format!("Failed to read config file: {}", String::from_utf8_lossy(&output.stderr)))
    }
}

// Tauri command to write config file
#[tauri::command]
async fn write_config(config_path: String, content: String, container_id: Option<u32>, vm_id: Option<u32>) -> Result<String, String> {
    let target = get_ssh_target(container_id, vm_id);
    
    // Create a backup first
    let backup_output = Command::new("ssh")
        .args([&target, "cp", &config_path, &format!("{}.backup", config_path)])
        .output();
    
    // Write the new content
    let output = Command::new("ssh")
        .args([&target, "tee", &config_path])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();
    
    match output {
        Ok(mut child) => {
            if let Some(stdin) = child.stdin.take() {
                use std::io::Write;
                let mut stdin = stdin;
                stdin.write_all(content.as_bytes()).map_err(|e| format!("Failed to write to stdin: {}", e))?;
            }
            
            let output = child.wait_with_output().map_err(|e| format!("Failed to wait for command: {}", e))?;
            
            if output.status.success() {
                Ok(format!("Config file {} updated successfully", config_path))
            } else {
                Err(format!("Failed to write config file: {}", String::from_utf8_lossy(&output.stderr)))
            }
        }
        Err(e) => Err(format!("Failed to spawn command: {}", e))
    }
}

// Helper functions for maintenance
async fn get_all_services() -> Result<Vec<ServiceInfo>, String> {
    let mut services = Vec::new();
    
    // Define services for each container/VM
    let service_definitions = [
        // Proxmox host services
        ("nginx", None, None),
        ("docker", None, None),
        ("ssh", None, None),
        
        // Container-specific services (these would be checked inside containers)
        ("sonarr", Some(214), None),
        ("radarr", Some(215), None),
        ("prowlarr", Some(210), None),
        ("qbittorrent", Some(212), None),
        ("plex", Some(230), None),
        ("jellyfin", Some(231), None),
        
        // VM services
        ("home-assistant", None, Some(500)),
        ("alexa-service", None, Some(611)),
    ];
    
    for &(service_name, container_id, vm_id) in &service_definitions {
        if let Ok(service_info) = check_service_status(service_name.to_string(), container_id, vm_id).await {
            services.push(service_info);
        }
    }
    
    Ok(services)
}

async fn get_all_binaries() -> Result<Vec<BinaryInfo>, String> {
    let mut binaries = Vec::new();
    
    // Define binaries to check
    let binary_definitions = [
        // System binaries
        ("docker", None, None),
        ("systemctl", None, None),
        ("nginx", None, None),
        
        // Application binaries (in containers)
        ("sonarr", Some(214), None),
        ("radarr", Some(215), None),
        ("prowlarr", Some(210), None),
        ("plex", Some(230), None),
        ("jellyfin", Some(231), None),
        
        // VM binaries
        ("python3", None, Some(500)),
        ("hass", None, Some(500)),
    ];
    
    for &(binary_name, container_id, vm_id) in &binary_definitions {
        if let Ok(binary_info) = check_binary(binary_name.to_string(), container_id, vm_id).await {
            binaries.push(binary_info);
        }
    }
    
    Ok(binaries)
}

async fn get_all_configs() -> Result<Vec<ConfigInfo>, String> {
    let mut configs = Vec::new();
    
    // Define config files to check
    let config_definitions = [
        // System configs
        ("/etc/nginx/nginx.conf", None, None),
        ("/etc/docker/daemon.json", None, None),
        
        // Application configs (in containers)
        ("/config/config.xml", Some(214), None), // Sonarr
        ("/config/config.xml", Some(215), None), // Radarr
        ("/config/config.xml", Some(210), None), // Prowlarr
        ("/config/qBittorrent/qBittorrent.conf", Some(212), None),
        
        // VM configs
        ("/config/configuration.yaml", None, Some(500)), // Home Assistant
    ];
    
    for &(config_path, container_id, vm_id) in &config_definitions {
        if let Ok(config_info) = check_config(config_path.to_string(), container_id, vm_id).await {
            configs.push(config_info);
        }
    }
    
    Ok(configs)
}

async fn get_system_health() -> Result<SystemHealth, String> {
    // Get system health from Proxmox host
    let df_output = Command::new("ssh")
        .args(["proxmox", "df", "-h", "/"])
        .output()
        .unwrap_or_default();
    
    let disk_usage = if df_output.status.success() {
        let df_str = String::from_utf8_lossy(&df_output.stdout);
        df_str.lines().nth(1)
            .and_then(|line| line.split_whitespace().nth(4))
            .and_then(|usage| usage.trim_end_matches('%').parse().ok())
            .unwrap_or(0.0)
    } else {
        0.0
    };
    
    let free_output = Command::new("ssh")
        .args(["proxmox", "free", "-m"])
        .output()
        .unwrap_or_default();
    
    let memory_usage = if free_output.status.success() {
        let free_str = String::from_utf8_lossy(&free_output.stdout);
        // Parse memory usage from free command output
        50.0 // Placeholder
    } else {
        0.0
    };
    
    let uptime_output = Command::new("ssh")
        .args(["proxmox", "uptime"])
        .output()
        .unwrap_or_default();
    
    let uptime = if uptime_output.status.success() {
        String::from_utf8_lossy(&uptime_output.stdout).trim().to_string()
    } else {
        "Unknown".to_string()
    };
    
    Ok(SystemHealth {
        disk_usage,
        memory_usage,
        cpu_load: 0.0, // Would need to parse from uptime or top
        network_status: "Connected".to_string(),
        uptime,
    })
}

fn get_ssh_target(container_id: Option<u32>, vm_id: Option<u32>) -> String {
    if let Some(cid) = container_id {
        // For containers, we might need to enter the container
        // This depends on your setup - you might use 'lxc exec' or docker exec
        format!("proxmox lxc exec {} --", cid)
    } else if let Some(vid) = vm_id {
        // For VMs, you might have separate SSH access
        match vid {
            500 => "homeassistant".to_string(), // Assuming you have SSH alias
            611 => "alexa".to_string(),
            900 => "ai-system".to_string(),
            _ => "proxmox".to_string(),
        }
    } else {
        "proxmox".to_string()
    }
}

impl Default for SystemHealth {
    fn default() -> Self {
        SystemHealth {
            disk_usage: 0.0,
            memory_usage: 0.0,
            cpu_load: 0.0,
            network_status: "Unknown".to_string(),
            uptime: "Unknown".to_string(),
        }
    }
}

// Tauri command to get system overview
#[tauri::command]
async fn get_system_overview() -> Result<SystemOverview, String> {
    let mut containers = Vec::new();
    let mut vms = Vec::new();

    // Get list of existing containers and VMs first
    let existing_container_ids = get_existing_containers().await.unwrap_or_default();
    let existing_vm_ids = get_existing_vms().await.unwrap_or_default();

    // Container definitions for metadata
    let container_metadata = [
        // Core Infrastructure (100-199)
        (100, "WireGuard", "VPN access and secure tunneling", "Core Infrastructure"),
        (101, "Gluetun", "VPN client container for other services", "Core Infrastructure"),
        (102, "Flaresolverr", "Cloudflare solver proxy", "Core Infrastructure"),
        (103, "Traefik", "Reverse proxy and load balancer", "Core Infrastructure"),
        (104, "Vaultwarden", "Password manager server", "Core Infrastructure"),
        (105, "Valkey", "Redis-compatible in-memory database", "Core Infrastructure"),
        (106, "PostgreSQL", "Primary database server", "Core Infrastructure"),
        (107, "Authentik", "Identity provider and SSO", "Core Infrastructure"),
        
        // Essential Media Services (210-229)
        (210, "Prowlarr", "Indexer manager and proxy", "Essential Media Services"),
        (211, "Jackett", "Torrent indexer proxy", "Essential Media Services"),
        (212, "QBittorrent", "BitTorrent client", "Essential Media Services"),
        (214, "Sonarr", "TV series management", "Essential Media Services"),
        (215, "Radarr", "Movie management", "Essential Media Services"),
        (216, "Proxarr", "Proxy management for *arr apps", "Essential Media Services"),
        (217, "Readarr", "Book and audiobook management", "Essential Media Services"),
        (219, "Whisparr", "Adult content management", "Essential Media Services"),
        (220, "Sonarr Extended", "Extended TV series management", "Essential Media Services"),
        (221, "Radarr Extended", "Extended movie management", "Essential Media Services"),
        (223, "Autobrr", "Automated torrent management", "Essential Media Services"),
        (224, "Deluge", "Alternative BitTorrent client", "Essential Media Services"),
        
        // Media Servers (230-239)
        (230, "Plex", "Media server and streaming platform", "Media Servers"),
        (231, "Jellyfin", "Open-source media server", "Media Servers"),
        (232, "Audiobookshelf", "Audiobook and podcast server", "Media Servers"),
        (233, "Calibre-web", "E-book server and manager", "Media Servers"),
        (234, "IPTV-Proxy", "IPTV streaming proxy", "Media Servers"),
        (235, "TVHeadend", "TV streaming server", "Media Servers"),
        (236, "Tdarr Server", "Media transcoding server", "Media Servers"),
        (237, "Tdarr Node", "Media transcoding worker", "Media Servers"),
        
        // Enhancement Services (240-250)
        (240, "Bazarr", "Subtitle management", "Enhancement Services"),
        (241, "Overseerr", "Media request management", "Enhancement Services"),
        (242, "Jellyseerr", "Jellyfin request management", "Enhancement Services"),
        (243, "Ombi", "Media request platform", "Enhancement Services"),
        (244, "Tautulli", "Plex monitoring and statistics", "Enhancement Services"),
        (245, "Kometa", "Plex metadata management", "Enhancement Services"),
        (246, "Gaps", "Plex collection gap finder", "Enhancement Services"),
        (247, "Janitorr", "Media cleanup automation", "Enhancement Services"),
        (248, "Decluttarr", "Media library decluttering", "Enhancement Services"),
        (249, "Watchlistarr", "Watchlist synchronization", "Enhancement Services"),
        (250, "Traktarr", "Trakt.tv integration", "Enhancement Services"),
        
        // Monitoring & Analytics (260-269)
        (260, "Prometheus", "Metrics collection and monitoring", "Monitoring & Analytics"),
        (261, "Grafana", "Metrics visualization and dashboards", "Monitoring & Analytics"),
        (262, "Checkrr", "Service health checking", "Monitoring & Analytics"),
        
        // Management & Utilities (270-279)
        (270, "FileBot", "File renaming and organization", "Management & Utilities"),
        (271, "FlexGet", "Automated content downloading", "Management & Utilities"),
        (272, "Buildarr", "Configuration management for *arr apps", "Management & Utilities"),
        (274, "Organizr", "Service organization dashboard", "Management & Utilities"),
        (275, "Homarr", "Modern dashboard for services", "Management & Utilities"),
        (276, "Homepage", "Customizable homepage dashboard", "Management & Utilities"),
        (277, "Recyclarr", "Configuration recycling for *arr apps", "Management & Utilities"),
        (278, "CrowdSec", "Collaborative security engine", "Management & Utilities"),
        (279, "Tailscale", "Secure networking mesh", "Management & Utilities"),
    ];

    // VM definitions for metadata
    let vm_metadata = [
        (500, "Home Assistant", "Home automation platform"),
        (611, "Alexa", "Voice assistant system"),
        (900, "AI System", "Artificial intelligence services"),
    ];

    // Only get status for containers that actually exist
    for &container_id in &existing_container_ids {
        if let Ok(mut container_info) = get_container_status(container_id).await {
            // Look up metadata for this container
            if let Some((_, name, description, category)) = container_metadata.iter().find(|(id, _, _, _)| *id == container_id) {
                container_info.name = name.to_string();
                container_info.category = category.to_string();
                container_info.description = description.to_string();
            } else {
                // Fallback for unknown containers
                container_info.name = format!("Container {}", container_id);
                container_info.category = get_container_category(container_id);
                container_info.description = "Unknown container".to_string();
            }
            containers.push(container_info);
        }
    }

    // Only get status for VMs that actually exist
    for &vm_id in &existing_vm_ids {
        if let Ok(mut vm_info) = get_vm_status(vm_id).await {
            // Look up metadata for this VM
            if let Some((_, name, description)) = vm_metadata.iter().find(|(id, _, _)| *id == vm_id) {
                vm_info.name = name.to_string();
                vm_info.description = description.to_string();
            } else {
                // Fallback for unknown VMs
                vm_info.name = format!("VM {}", vm_id);
                vm_info.description = "Unknown virtual machine".to_string();
            }
            vms.push(vm_info);
        }
    }

    let running_containers = containers.iter().filter(|c| c.status == "Running").count() as u32;
    let running_vms = vms.iter().filter(|v| v.status == "Running").count() as u32;

    Ok(SystemOverview {
        total_containers: containers.len() as u32,
        running_containers,
        total_vms: vms.len() as u32,
        running_vms,
        containers,
        vms,
        last_updated: Utc::now(),
    })
}

// Helper functions
async fn get_container_info(container_id: u32) -> Result<ContainerInfo, String> {
    // This would typically fetch detailed container information
    // For now, return basic info
    Ok(ContainerInfo {
        id: container_id,
        name: format!("Container {}", container_id),
        status: "Unknown".to_string(),
        uptime: "Unknown".to_string(),
        cpu_usage: 0.0,
        memory_usage: 0.0,
        category: get_container_category(container_id),
        description: get_container_description(container_id),
    })
}

fn get_container_category(container_id: u32) -> String {
    match container_id {
        100..=199 => "Core Infrastructure".to_string(),
        210..=229 => "Essential Media Services".to_string(),
        230..=239 => "Media Servers".to_string(),
        240..=250 => "Enhancement Services".to_string(),
        260..=269 => "Monitoring & Analytics".to_string(),
        270..=279 => "Management & Utilities".to_string(),
        _ => "Other".to_string(),
    }
}

fn get_container_description(container_id: u32) -> String {
    match container_id {
        100 => "VPN access and secure tunneling".to_string(),
        101 => "VPN client container for other services".to_string(),
        102 => "Cloudflare solver proxy".to_string(),
        103 => "Reverse proxy and load balancer".to_string(),
        104 => "Password manager server".to_string(),
        105 => "Redis-compatible in-memory database".to_string(),
        106 => "Primary database server".to_string(),
        107 => "Identity provider and SSO".to_string(),
        _ => "Service container".to_string(),
    }
}

fn get_vm_name(vm_id: u32) -> String {
    match vm_id {
        500 => "Home Assistant".to_string(),
        611 => "Alexa".to_string(),
        900 => "AI System".to_string(),
        _ => format!("VM {}", vm_id),
    }
}

fn get_vm_description(vm_id: u32) -> String {
    match vm_id {
        500 => "Home automation platform".to_string(),
        611 => "Voice assistant system".to_string(),
        900 => "Artificial intelligence services".to_string(),
        _ => "Virtual machine".to_string(),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_container_status,
            get_vm_status,
            start_container,
            stop_container,
            restart_container,
            start_vm,
            stop_vm,
            get_system_overview,
            get_maintenance_overview,
            check_service_status,
            control_service,
            check_binary,
            check_config,
            read_config,
            write_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

