//! Drive the packed component through `act run --mcp` with a real MCP client.
//!
//! This replaces the python fastmcp/pytest suite that used to live in this
//! directory: the tests observe exactly what an agent observes, over the same
//! client stack (`rmcp`) the host bridge itself is built on.
//!
//! Env: WASM — path to the packed component (default: the component's
//!      release build output);
//!      ACT  — the act invocation (default `act`; `npx @actcore/act`, the
//!             component justfile's default, also works — whitespace-split,
//!             like the shlex.split the python conftest did).

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use rmcp::{ServiceExt, model::CallToolRequestParams, transport::TokioChildProcess};
use regex::Regex;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex as AsyncMutex;

/// `().serve(transport)` hands back the client-role service running over the
/// child process: role first, the unit client handler second.
type Client = rmcp::service::RunningService<rmcp::service::RoleClient, ()>;

fn wasm_path() -> PathBuf {
    PathBuf::from(std::env::var("WASM").unwrap_or_else(|_| {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../target/wasm32-wasip2/release/component_random.wasm"
        )
        .into()
    }))
}

/// The ACT invocation, honouring the same override the component justfile
/// uses. Its default there is `npx @actcore/act` — two words — which cannot
/// be `argv[0]` for a non-shell spawn, so the value is whitespace-split into
/// program + leading args. Quoted paths with spaces are not a form this
/// fleet passes through `ACT`; a full shlex is deliberately not pulled in.
fn act_argv() -> Vec<String> {
    std::env::var("ACT")
        .unwrap_or_else(|_| "act".into())
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

/// Spawn `act run <wasm> --mcp` with no grant flags: random declares no
/// capabilities (pure computation — no filesystem, network, or sockets), so
/// there is nothing to grant and nothing to refuse.
fn act_command() -> tokio::process::Command {
    let argv = act_argv();
    let mut cmd = tokio::process::Command::new(&argv[0]);
    cmd.args(&argv[1..]);
    cmd.arg("run").arg(wasm_path()).arg("--mcp");
    cmd
}

fn spawn_transport() -> TokioChildProcess {
    TokioChildProcess::new(act_command()).expect("spawn act run --mcp")
}

/// Spawn with stderr captured: the audit trail (refusals, per-call rollup)
/// writes there unconditionally — RUST_LOG never silences it.
fn spawn_with_captured_stderr() -> (TokioChildProcess, Arc<AsyncMutex<String>>) {
    let (transport, stderr) = TokioChildProcess::builder(act_command())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn act run --mcp with piped stderr");

    let captured = Arc::new(AsyncMutex::new(String::new()));
    let sink = captured.clone();
    let mut lines = BufReader::new(stderr.expect("stderr was piped")).lines();
    tokio::spawn(async move {
        while let Ok(Some(line)) = lines.next_line().await {
            sink.lock().await.push_str(&line);
            sink.lock().await.push('\n');
        }
    });

    (transport, captured)
}

/// Poll the captured stderr until `needle` appears — the audit line is
/// flushed before the JSON-RPC reply, but reaching this buffer still crosses
/// a pipe and an async read.
async fn wait_for_stderr(
    captured: &Arc<AsyncMutex<String>>,
    needle: &str,
    timeout: Duration,
) -> bool {
    let start = std::time::Instant::now();
    loop {
        if captured.lock().await.contains(needle) {
            return true;
        }
        if start.elapsed() > timeout {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

async fn connect() -> Client {
    ().serve(spawn_transport())
        .await
        .expect("rmcp handshake with act run --mcp")
}

fn first_text_block(result: &rmcp::model::CallToolResult) -> &rmcp::model::TextContent {
    match result.content.first() {
        Some(rmcp::model::ContentBlock::Text(t)) => t,
        other => panic!("expected the first content block to be Text, got: {other:?}"),
    }
}

/// The python suite's `re.fullmatch`, which has no direct `regex` equivalent:
/// a bare `is_match` would accept a partial match. Translated assertions must
/// not silently loosen, so the match is required to span the whole string.
fn assert_full_match(text: &str, pattern: &str) {
    let re = Regex::new(pattern).expect("test regex compiles");
    match re.find(text) {
        None => panic!("`{text}` does not match {pattern}"),
        Some(m) if m.as_str() == text => {}
        Some(m) => panic!("`{text}` only partially matches {pattern} (matched `{}`)", m.as_str()),
    }
}

/// The kind and message of a failed call may arrive on either path: as a
/// JSON-RPC error response (`ErrorData.data`) or as an isError result
/// (`_meta`). The python conftest's `expect_error` fixture handled both; so
/// does this. `call-tool` has no `result<>` wrapper, so a guest reporting a
/// failed call can only do it through `tool-event::error` — which is the
/// isError path here; the JSON-RPC path stays handled for the non-guest
/// failure modes (list-tools, session ops, wasmtime trap, unreachable actor).
async fn error_kind_of(client: &Client, params: CallToolRequestParams) -> Option<String> {
    match client.call_tool(params).await {
        Err(rmcp::ServiceError::McpError(e)) => e
            .data
            .as_ref()
            .and_then(|d| d.get("dev.actcore/error-kind"))
            .and_then(|v| v.as_str())
            .map(str::to_string),
        Ok(result) => {
            assert_eq!(result.is_error, Some(true), "call must fail: {result:?}");
            result
                .meta
                .as_ref()
                .and_then(|m| m.0.get("dev.actcore/error-kind"))
                .and_then(|v| v.as_str())
                .map(str::to_string)
        }
        Err(other) => panic!("unexpected transport failure: {other:?}"),
    }
}

/// Call a tool that must succeed and return its single text block.
async fn call_ok(client: &Client, tool: &str, args: Value) -> String {
    let result = client
        .call_tool(CallToolRequestParams::new(tool.to_string()).with_arguments(
            args.as_object().expect("args are a JSON object").clone(),
        ))
        .await
        .unwrap_or_else(|e| panic!("call_tool {tool}: {e:?}"));
    assert_ne!(result.is_error, Some(true), "{tool} failed: {result:?}");
    first_text_block(&result).text.clone()
}

/// The manifest probe from the python test_info.py: the packed artifact
/// must declare its name and a version. Also the fast-fail the python
/// `wasm_path` fixture provided — an unpacked wasm (raw `cargo build`
/// output, no `act:component` section) declares no ceiling, every grant is
/// refused as "outside ceiling", and the failures point anywhere but at the
/// missing metadata. The justfile's `test: build` ordering exists so this
/// test finds a packed artifact.
#[test]
fn manifest_reports_name_and_version() {
    let output = {
        let argv = act_argv();
        let mut cmd = std::process::Command::new(&argv[0]);
        cmd.args(&argv[1..]);
        cmd.args(["inspect", "component-manifest"])
            .arg(wasm_path())
            .output()
            .expect("run act inspect component-manifest")
    };
    assert!(
        output.status.success(),
        "inspect failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let manifest: Value = serde_json::from_slice(&output.stdout).expect("manifest is JSON");
    assert_eq!(
        manifest["std"]["name"], "random",
        "packed manifest must carry the component name"
    );
    assert!(
        manifest["std"]["version"].is_string(),
        "packed manifest must carry a version, got: {}",
        manifest["std"]["version"]
    );
}

#[tokio::test]
async fn component_exposes_its_tools() {
    let client = connect().await;
    let tools = client.list_all_tools().await.expect("list_all_tools");
    assert!(
        !tools.is_empty(),
        "component must expose at least one tool, got: {:?}",
        tools.iter().map(|t| t.name.to_string()).collect::<Vec<_>>()
    );
    client.cancel().await.ok();
}

/// python test_random_number.py CASES: (args, expected-regex).
#[tokio::test]
async fn random_number_matches_range_shape() {
    let client = connect().await;
    let cases = [
        // default range 0-100
        (json!({}), r"^-?\d+$"),
        (json!({"min": 1, "max": 6}), r"^[1-6]$"),
    ];
    for (args, pattern) in &cases {
        let text = call_ok(&client, "random_number", args.clone()).await;
        assert_full_match(&text, pattern);
    }
    client.cancel().await.ok();
}

/// A schema-valid but logically inverted range trips the guest's own check
/// (`min must be <= max`), reported as std:invalid-args.
#[tokio::test]
async fn rejects_an_invalid_range() {
    let client = connect().await;
    let kind = error_kind_of(
        &client,
        CallToolRequestParams::new("random_number").with_arguments(
            json!({"min": 10, "max": 1})
                .as_object()
                .unwrap()
                .clone(),
        ),
    )
    .await
    .expect("the failure must carry a named error kind");
    assert_eq!(kind, "std:invalid-args");
    client.cancel().await.ok();
}

/// python test_random_string.py CASES: (args, expected-regex).
#[tokio::test]
async fn random_string_matches_charset_shape() {
    let client = connect().await;
    let cases = [
        // default alphanumeric
        (json!({"length": 16}), r"^[a-zA-Z0-9]{16}$"),
        (json!({"length": 8, "charset": "hex"}), r"^[0-9a-f]{8}$"),
        (json!({"length": 6, "charset": "digits"}), r"^[0-9]{6}$"),
    ];
    for (args, pattern) in &cases {
        let text = call_ok(&client, "random_string", args.clone()).await;
        assert_full_match(&text, pattern);
    }
    client.cancel().await.ok();
}

/// python test_uuid.py CASES: (version-or-absent, expected-regex).
#[tokio::test]
async fn uuid_matches_version_shape() {
    let client = connect().await;
    let uuid_v4 = r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$";
    let uuid_v7 = r"^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$";
    let cases = [
        (json!({}), uuid_v4), // v4 (default)
        (json!({"version": 7}), uuid_v7),
    ];
    for (args, pattern) in &cases {
        let text = call_ok(&client, "uuid", args.clone()).await;
        assert_full_match(&text, pattern);
    }
    client.cancel().await.ok();
}

/// An unsupported version is refused with std:invalid-args — the host schema
/// check first (ACT-SPEC §6.4), the guest's own `Use 4 or 7.` check behind it.
#[tokio::test]
async fn rejects_an_unsupported_version() {
    let client = connect().await;
    let kind = error_kind_of(
        &client,
        CallToolRequestParams::new("uuid").with_arguments(
            json!({"version": 3}).as_object().unwrap().clone(),
        ),
    )
    .await
    .expect("the failure must carry a named error kind");
    assert_eq!(kind, "std:invalid-args");
    client.cancel().await.ok();
}

/// Beyond python parity: the audit machinery. A call must leave its rollup
/// line on stderr, and the captured-stderr plumbing this harness uses for
/// refusals must actually see it.
#[tokio::test]
async fn random_call_is_audited() {
    let (transport, captured) = spawn_with_captured_stderr();
    let client = ().serve(transport).await.expect("rmcp handshake");

    let text = call_ok(&client, "uuid", json!({})).await;
    assert_full_match(&text, r"^[0-9a-f-]{36}$");

    assert!(
        wait_for_stderr(&captured, "req:", Duration::from_secs(5)).await,
        "expected a per-call rollup line in the audit trail:\n{}",
        captured.lock().await
    );

    client.cancel().await.ok();
}
