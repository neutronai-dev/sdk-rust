use neutron_ai::mcp::{
    create_anthropic_mcp_connector, create_neutron_mcp_server_url, create_openai_mcp_tool,
};

#[test]
fn normalizes_mcp_url() {
    assert_eq!(
        create_neutron_mcp_server_url("https://mcp.neutronai.dev/", Some("mcp")),
        "https://mcp.neutronai.dev/mcp"
    );
}

#[test]
fn builds_provider_mcp_shapes() {
    let openai = create_openai_mcp_tool("https://mcp.neutronai.dev/mcp", None);
    assert_eq!(openai["server_label"], "neutron_memory");

    let anthropic =
        create_anthropic_mcp_connector("https://mcp.neutronai.dev/mcp", None, Some("token"));
    assert_eq!(anthropic["mcp_servers"][0]["authorization_token"], "token");
}
