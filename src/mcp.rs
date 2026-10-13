use serde_json::{Value, json};

pub fn create_neutron_mcp_server_url(base_url: &str, path: Option<&str>) -> String {
    let normalized_path = path.unwrap_or("/mcp");
    let normalized_path = if normalized_path.starts_with('/') {
        normalized_path.to_string()
    } else {
        format!("/{normalized_path}")
    };
    format!("{}{}", base_url.trim_end_matches('/'), normalized_path)
}

pub fn create_openai_mcp_tool(server_url: &str, server_label: Option<&str>) -> Value {
    json!({
        "type": "mcp",
        "server_label": server_label.unwrap_or("neutron_memory"),
        "server_url": server_url,
        "require_approval": "never"
    })
}

pub fn create_anthropic_mcp_connector(
    server_url: &str,
    server_label: Option<&str>,
    authorization: Option<&str>,
) -> Value {
    let server_label = server_label.unwrap_or("neutron_memory");
    let mut server = json!({
        "type": "url",
        "url": server_url,
        "name": server_label
    });
    if let Some(authorization) = authorization {
        server["authorization_token"] = json!(authorization);
    }

    json!({
        "mcp_servers": [server],
        "tools": [{
            "type": "mcp_toolset",
            "mcp_server_name": server_label
        }],
        "betas": ["mcp-client-2025-11-20"]
    })
}
