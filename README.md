# neutron-ai

Rust SDK for Neutron AI memory and MCP operations. Rust 1.85 or newer is required.

## Install

```toml
[dependencies]
neutron-ai = "0.2"
```

## Usage

```rust
let client = neutron_ai::NeutronAIClient::new(neutron_ai::ClientOptions {
    base_url: "https://neutronai.dev/api".to_string(),
    token: Some("your_token".to_string()),
    nucleus_id: Some("nucleus_demo".to_string()),
    tenant_id: None,
})?;
```

The SDK covers the core memory lifecycle, audit and cache operations, and MCP tool discovery. `tenant_id` remains a compatibility alias and must match `nucleus_id` when both are provided.
