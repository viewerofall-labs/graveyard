mod lua;

use std::path::Path;
use std::sync::Arc;
use std::os::unix::fs::PermissionsExt;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{error, info};
use zbus::Connection;

use shared::{Command, Response, ServiceInfo, ServiceStatus};

const SOCKET_PATH: &str = "/tmp/systemd-tui.sock";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    if Path::new(SOCKET_PATH).exists() {
        std::fs::remove_file(SOCKET_PATH).expect("Failed to remove old socket");
    }

    let lua_engine = lua::LuaEngine::new().expect("Failed to init Lua");
    lua_engine.load_config().expect("Failed to load Lua config");
    let lua_engine = Arc::new(lua_engine);

    let connection = Connection::system()
        .await
        .expect("Failed to connect to D-Bus");
    let connection = Arc::new(connection);

    let listener = UnixListener::bind(SOCKET_PATH).expect("Failed to bind socket");
    // Make socket accessible to all users so the client can connect without sudo
    std::fs::set_permissions(
        SOCKET_PATH,
        std::os::unix::fs::PermissionsExt::from_mode(0o666),
    ).expect("Failed to set socket permissions");
    info!("Daemon listening on {}", SOCKET_PATH);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let conn = connection.clone();
                let lua = lua_engine.clone();
                tokio::spawn(handle_client(stream, conn, lua));
            }
            Err(e) => error!("Failed to accept connection: {}", e),
        }
    }
}

async fn handle_client(
    stream: UnixStream,
    connection: Arc<Connection>,
    lua_engine: Arc<lua::LuaEngine>,
) {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break,
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                let response = match serde_json::from_str::<Command>(trimmed) {
                    Ok(cmd) => handle_command(cmd, &connection, &lua_engine).await,
                    Err(e) => Response::Error(format!("Invalid command: {}", e)),
                };

                let mut json = serde_json::to_string(&response).unwrap();
                json.push('\n');

                if let Err(e) = writer.write_all(json.as_bytes()).await {
                    error!("Failed to write response: {}", e);
                    break;
                }
            }
            Err(e) => {
                error!("Read error: {}", e);
                break;
            }
        }
    }
}

async fn handle_command(
    cmd: Command,
    connection: &Connection,
    lua_engine: &lua::LuaEngine,
) -> Response {
    info!("Received command: {:?}", cmd);
    match cmd {
        Command::ListServices => match list_services(connection).await {
            Ok(services) => {
                let filtered = services
                    .into_iter()
                    .filter(|s| !lua_engine.is_hidden(&s.name))
                    .collect();
                let groups = lua_engine.get_groups();
                Response::ServiceList {
                    services: filtered,
                    groups,
                }
            }
            Err(e) => Response::Error(format!("Failed to list services: {}", e)),
        },
        Command::StartService(name) => {
            match control_service(connection, &name, "StartUnit").await {
                Ok(_) => Response::Success(format!("Started {}", name)),
                Err(e) => Response::Error(format!("Failed to start {}: {}", name, e)),
            }
        }
        Command::StopService(name) => match control_service(connection, &name, "StopUnit").await {
            Ok(_) => Response::Success(format!("Stopped {}", name)),
            Err(e) => Response::Error(format!("Failed to stop {}: {}", name, e)),
        },
        Command::RestartService(name) => {
            match control_service(connection, &name, "RestartUnit").await {
                Ok(_) => Response::Success(format!("Restarted {}", name)),
                Err(e) => Response::Error(format!("Failed to restart {}: {}", name, e)),
            }
        }
        Command::GetStatus(name) => match get_service_status(connection, &name).await {
            Ok(info) => Response::ServiceStatus(info),
            Err(e) => Response::Error(format!("Failed to get status: {}", e)),
        },
        Command::GetDetailedStatus(name) => {
            match tokio::process::Command::new("systemctl")
                .args(["status", "--no-pager", "--full", &name])
                .output()
                .await
            {
                Ok(output) => {
                    let text = String::from_utf8_lossy(&output.stdout).to_string();
                    Response::DetailedStatus(text)
                }
                Err(e) => Response::Error(format!("Failed to run systemctl status: {}", e)),
            }
        }
        Command::EnableService(name) => {
            match tokio::process::Command::new("systemctl")
                .args(["enable", &name])
                .output()
                .await
            {
                Ok(_) => Response::Success(format!("Enabled {}", name)),
                Err(e) => Response::Error(format!("Failed to enable {}: {}", name, e)),
            }
        }
        Command::DisableService(name) => {
            match tokio::process::Command::new("systemctl")
                .args(["disable", &name])
                .output()
                .await
            {
                Ok(_) => Response::Success(format!("Disabled {}", name)),
                Err(e) => Response::Error(format!("Failed to disable {}: {}", name, e)),
            }
        }
    }
}

async fn list_services(connection: &Connection) -> anyhow::Result<Vec<ServiceInfo>> {
    let message = connection
        .call_method(
            Some("org.freedesktop.systemd1"),
            "/org/freedesktop/systemd1",
            Some("org.freedesktop.systemd1.Manager"),
            "ListUnits",
            &(),
        )
        .await?;

    let units: Vec<(
        String,
        String,
        String,
        String,
        String,
        String,
        zbus::zvariant::OwnedObjectPath,
        u32,
        String,
        zbus::zvariant::OwnedObjectPath,
    )> = message.body().deserialize()?;

    // Get enabled state for all units in one shot
    let enabled_map = get_enabled_map().await.unwrap_or_default();

    let services = units
        .into_iter()
        .filter(|(name, _, _, _, _, _, _, _, _, _)| name.ends_with(".service"))
        .map(|(name, description, _, active_state, _, _, _, _, _, _)| {
            let status = match active_state.as_str() {
                "active" => ServiceStatus::Active,
                "inactive" => ServiceStatus::Inactive,
                "failed" => ServiceStatus::Failed,
                _ => ServiceStatus::Unknown,
            };
            let enabled = enabled_map.get(&name).copied().unwrap_or(false);
            ServiceInfo {
                name,
                status,
                description,
                enabled,
            }
        })
        .collect();

    Ok(services)
}

async fn get_enabled_map() -> anyhow::Result<std::collections::HashMap<String, bool>> {
    let output = tokio::process::Command::new("systemctl")
        .args([
            "list-unit-files",
            "--type=service",
            "--no-pager",
            "--no-legend",
        ])
        .output()
        .await?;

    let mut map = std::collections::HashMap::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let mut parts = line.split_whitespace();
        if let (Some(name), Some(state)) = (parts.next(), parts.next()) {
            map.insert(
                name.to_string(),
                matches!(state, "enabled" | "enabled-runtime"),
            );
        }
    }
    Ok(map)
}

async fn control_service(connection: &Connection, name: &str, method: &str) -> anyhow::Result<()> {
    connection
        .call_method(
            Some("org.freedesktop.systemd1"),
            "/org/freedesktop/systemd1",
            Some("org.freedesktop.systemd1.Manager"),
            method,
            &(name, "replace"),
        )
        .await?;
    Ok(())
}

async fn get_service_status(connection: &Connection, name: &str) -> anyhow::Result<ServiceInfo> {
    let message = connection
        .call_method(
            Some("org.freedesktop.systemd1"),
            "/org/freedesktop/systemd1",
            Some("org.freedesktop.systemd1.Manager"),
            "GetUnit",
            &name,
        )
        .await?;

    let path: zbus::zvariant::OwnedObjectPath = message.body().deserialize()?;

    let active_state: String = connection
        .call_method(
            Some("org.freedesktop.systemd1"),
            path.as_str(),
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.freedesktop.systemd1.Unit", "ActiveState"),
        )
        .await?
        .body()
        .deserialize::<zbus::zvariant::Value>()
        .map(|v| v.to_string())?;

    let status = match active_state.as_str() {
        "active" => ServiceStatus::Active,
        "inactive" => ServiceStatus::Inactive,
        "failed" => ServiceStatus::Failed,
        _ => ServiceStatus::Unknown,
    };

    Ok(ServiceInfo {
        name: name.to_string(),
        status,
        description: String::new(),
        enabled: false,
    })
}
