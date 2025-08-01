use serde::{Deserialize, Serialize};
use std::process::Command;
use tauri::State;
use tokio::time::{sleep, Duration};
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

// Tauri command to get container status
#[tauri::command]
async fn get_container_status(container_id: u32) -> Result<ContainerInfo, String> {
    let output = Command::new("pct")
        .args(["status", &container_id.to_string()])
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

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

    let container_info = get_container_info(container_id).await.unwrap_or_else(|_| {
        ContainerInfo {
            id: container_id,
            name: format!("Container {}", container_id),
            status,
            uptime: "Unknown".to_string(),
            cpu_usage: 0.0,
            memory_usage: 0.0,
            category: get_container_category(container_id),
            description: get_container_description(container_id),
        }
    });

    Ok(container_info)
}

// Tauri command to get VM status
#[tauri::command]
async fn get_vm_status(vm_id: u32) -> Result<VMInfo, String> {
    let output = Command::new("qm")
        .args(["status", &vm_id.to_string()])
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

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
    let output = Command::new("pct")
        .args(["start", &container_id.to_string()])
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok(format!("Container {} started successfully", container_id))
    } else {
        Err(format!("Failed to start container {}: {}", container_id, String::from_utf8_lossy(&output.stderr)))
    }
}

// Tauri command to stop container
#[tauri::command]
async fn stop_container(container_id: u32) -> Result<String, String> {
    let output = Command::new("pct")
        .args(["stop", &container_id.to_string()])
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok(format!("Container {} stopped successfully", container_id))
    } else {
        Err(format!("Failed to stop container {}: {}", container_id, String::from_utf8_lossy(&output.stderr)))
    }
}

// Tauri command to restart container
#[tauri::command]
async fn restart_container(container_id: u32) -> Result<String, String> {
    let output = Command::new("pct")
        .args(["restart", &container_id.to_string()])
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok(format!("Container {} restarted successfully", container_id))
    } else {
        Err(format!("Failed to restart container {}: {}", container_id, String::from_utf8_lossy(&output.stderr)))
    }
}

// Tauri command to start VM
#[tauri::command]
async fn start_vm(vm_id: u32) -> Result<String, String> {
    let output = Command::new("qm")
        .args(["start", &vm_id.to_string()])
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok(format!("VM {} started successfully", vm_id))
    } else {
        Err(format!("Failed to start VM {}: {}", vm_id, String::from_utf8_lossy(&output.stderr)))
    }
}

// Tauri command to stop VM
#[tauri::command]
async fn stop_vm(vm_id: u32) -> Result<String, String> {
    let output = Command::new("qm")
        .args(["stop", &vm_id.to_string()])
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok(format!("VM {} stopped successfully", vm_id))
    } else {
        Err(format!("Failed to stop VM {}: {}", vm_id, String::from_utf8_lossy(&output.stderr)))
    }
}

// Tauri command to get system overview
#[tauri::command]
async fn get_system_overview() -> Result<SystemOverview, String> {
    let mut containers = Vec::new();
    let mut vms = Vec::new();

    // Core Infrastructure (100-199)
    let core_containers = vec![
        (100, "WireGuard", "VPN access and secure tunneling"),
        (101, "Gluetun", "VPN client container for other services"),
        (102, "Flaresolverr", "Cloudflare solver proxy"),
        (103, "Traefik", "Reverse proxy and load balancer"),
        (104, "Vaultwarden", "Password manager server"),
        (105, "Valkey", "Redis-compatible in-memory database"),
        (106, "PostgreSQL", "Primary database server"),
        (107, "Authentik", "Identity provider and SSO"),
    ];

    // Essential Media Services (210-229)
    let media_containers = vec![
        (210, "Prowlarr", "Indexer manager and proxy"),
        (211, "Jackett", "Torrent indexer proxy"),
        (212, "QBittorrent", "BitTorrent client"),
        (214, "Sonarr", "TV series management"),
        (215, "Radarr", "Movie management"),
        (216, "Proxarr", "Proxy management for *arr apps"),
        (217, "Readarr", "Book and audiobook management"),
        (219, "Whisparr", "Adult content management"),
        (220, "Sonarr Extended", "Extended TV series management"),
        (221, "Radarr Extended", "Extended movie management"),
        (223, "Autobrr", "Automated torrent management"),
        (224, "Deluge", "Alternative BitTorrent client"),
    ];

    // Media Servers (230-239)
    let server_containers = vec![
        (230, "Plex", "Media server and streaming platform"),
        (231, "Jellyfin", "Open-source media server"),
        (232, "Audiobookshelf", "Audiobook and podcast server"),
        (233, "Calibre-web", "E-book server and manager"),
        (234, "IPTV-Proxy", "IPTV streaming proxy"),
        (235, "TVHeadend", "TV streaming server"),
        (236, "Tdarr Server", "Media transcoding server"),
        (237, "Tdarr Node", "Media transcoding worker"),
    ];

    // Enhancement Services (240-249)
    let enhancement_containers = vec![
        (240, "Bazarr", "Subtitle management"),
        (241, "Overseerr", "Media request management"),
        (242, "Jellyseerr", "Jellyfin request management"),
        (243, "Ombi", "Media request platform"),
        (244, "Tautulli", "Plex monitoring and statistics"),
        (245, "Kometa", "Plex metadata management"),
        (246, "Gaps", "Plex collection gap finder"),
        (247, "Janitorr", "Media cleanup automation"),
        (248, "Decluttarr", "Media library decluttering"),
        (249, "Watchlistarr", "Watchlist synchronization"),
        (250, "Traktarr", "Trakt.tv integration"),
    ];

    // Monitoring & Analytics (260-269)
    let monitoring_containers = vec![
        (260, "Prometheus", "Metrics collection and monitoring"),
        (261, "Grafana", "Metrics visualization and dashboards"),
        (262, "Checkrr", "Service health checking"),
    ];

    // Management & Utilities (270-279)
    let utility_containers = vec![
        (270, "FileBot", "File renaming and organization"),
        (271, "FlexGet", "Automated content downloading"),
        (272, "Buildarr", "Configuration management for *arr apps"),
        (274, "Organizr", "Service organization dashboard"),
        (275, "Homarr", "Modern dashboard for services"),
        (276, "Homepage", "Customizable homepage dashboard"),
        (277, "Recyclarr", "Configuration recycling for *arr apps"),
        (278, "CrowdSec", "Collaborative security engine"),
        (279, "Tailscale", "Secure networking mesh"),
    ];

    // Combine all container definitions
    let all_containers = vec![
        (core_containers, "Core Infrastructure"),
        (media_containers, "Essential Media Services"),
        (server_containers, "Media Servers"),
        (enhancement_containers, "Enhancement Services"),
        (monitoring_containers, "Monitoring & Analytics"),
        (utility_containers, "Management & Utilities"),
    ];

    // Get status for all containers
    for (container_group, category) in all_containers {
        for (id, name, description) in container_group {
            if let Ok(mut container_info) = get_container_status(id).await {
                container_info.name = name.to_string();
                container_info.category = category.to_string();
                container_info.description = description.to_string();
                containers.push(container_info);
            }
        }
    }

    // Get VM information
    let vm_definitions = vec![
        (500, "Home Assistant", "Home automation platform"),
        (611, "Alexa", "Voice assistant system"),
        (900, "AI System", "Artificial intelligence services"),
    ];

    for (id, name, description) in vm_definitions {
        if let Ok(mut vm_info) = get_vm_status(id).await {
            vm_info.name = name.to_string();
            vm_info.description = description.to_string();
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
            get_system_overview
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

