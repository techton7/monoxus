//! `wasm32` WebMCP bridge exposing strictly `["snapshot", "query", "act"]`
//! (`FM-6`: `resume_approval` is NEVER registered to untrusted browser agents).

use std::collections::BTreeMap;

use ui_sdk::agent::{
    ActionCall, AgentReply, AgentRequest, AgentToolDescriptor, SearchFilter, SnapshotValue,
    StepOutcome, TargetQuery, agent_tool_descriptors,
};

use super::runtime::AgentBridgeHandle;

/// Strictly allowlisted WebMCP tool names (`FM-6`: excludes `"resume_approval"`).
pub const WEBMCP_TOOL_NAMES: [&str; 3] = ["snapshot", "query", "act"];

/// Returns the static tool descriptors permitted for WebMCP registration.
///
/// Strictly filters [`agent_tool_descriptors`] to [`WEBMCP_TOOL_NAMES`], guaranteeing
/// that `"resume_approval"` is never exposed to untrusted browser agents.
pub fn webmcp_tool_descriptors() -> Vec<&'static AgentToolDescriptor> {
    agent_tool_descriptors()
        .iter()
        .filter(|descriptor| WEBMCP_TOOL_NAMES.contains(&descriptor.name))
        .collect()
}

pub(crate) fn translate_webmcp_tool_call(
    tool: &str,
    input: &oxidase::serde_json::Value,
) -> Result<AgentRequest, String> {
    if !WEBMCP_TOOL_NAMES.contains(&tool) {
        return Err(format!("tool '{tool}' is not permitted via WebMCP"));
    }

    match tool {
        "snapshot" => {
            let include_off_page = input
                .get("include_off_page")
                .and_then(oxidase::serde_json::Value::as_bool)
                .unwrap_or(true);
            Ok(AgentRequest::Snapshot { include_off_page })
        }
        "query" => {
            if let Some(target_val) = input.get("target") {
                let target = parse_target_query(target_val)?;
                if let (Some(field), Some(expected_val)) = (
                    input
                        .get("field")
                        .and_then(oxidase::serde_json::Value::as_str),
                    input.get("expected"),
                ) {
                    let expected: SnapshotValue =
                        oxidase::serde_json::from_value(expected_val.clone())
                            .map_err(|err| format!("invalid expected value: {err}"))?;
                    return Ok(AgentRequest::Expect {
                        target,
                        field: field.to_owned(),
                        expected,
                    });
                }
                return Ok(AgentRequest::Query { target });
            }

            let filter: SearchFilter = if input.is_null() {
                SearchFilter::new()
            } else {
                oxidase::serde_json::from_value(input.clone())
                    .map_err(|err| format!("invalid query filter: {err}"))?
            };
            Ok(AgentRequest::Search(filter))
        }
        "act" => {
            let target_val = input
                .get("target")
                .ok_or_else(|| String::from("missing required 'target' field"))?;
            let target = parse_target_query(target_val)?;

            let call = if let Some(call_val) = input.get("call") {
                oxidase::serde_json::from_value::<ActionCall>(call_val.clone())
                    .map_err(|err| format!("invalid action call: {err}"))?
            } else {
                let action_name = input
                    .get("action")
                    .and_then(oxidase::serde_json::Value::as_str)
                    .ok_or_else(|| String::from("missing required 'action' field"))?;
                let args: BTreeMap<String, SnapshotValue> = match input.get("args") {
                    Some(val) if !val.is_null() => oxidase::serde_json::from_value(val.clone())
                        .map_err(|err| format!("invalid action args: {err}"))?,
                    _ => BTreeMap::new(),
                };
                ActionCall::with_args(action_name, args)
            };

            Ok(AgentRequest::Act { target, call })
        }
        _ => Err(format!("tool '{tool}' is not permitted via WebMCP")),
    }
}

fn parse_target_query(value: &oxidase::serde_json::Value) -> Result<TargetQuery, String> {
    if let Some(raw) = value.as_str() {
        return Ok(TargetQuery::parse(raw));
    }
    oxidase::serde_json::from_value(value.clone())
        .map_err(|err| format!("invalid target query: {err}"))
}

#[allow(dead_code)]
pub(crate) async fn execute_webmcp_call(
    handle: &AgentBridgeHandle,
    tool: &str,
    input: &oxidase::serde_json::Value,
) -> oxidase::serde_json::Value {
    let request = match translate_webmcp_tool_call(tool, input) {
        Ok(req) => req,
        Err(message) => {
            return oxidase::serde_json::json!({
                "ok": false,
                "error": message,
            });
        }
    };

    let reply = handle.handle_request_settled(request).await;

    if let AgentReply::Step(StepOutcome::NeedsApproval {
        approval_id,
        request: approval_request,
        ..
    }) = reply
    {
        if let Some(handler) = handle.approval_handler() {
            let decision = (handler)(approval_request).await;
            let resumed = handle
                .handle_request_settled(AgentRequest::ResumeApproval {
                    approval_id,
                    decision,
                })
                .await;
            return oxidase::serde_json::json!({
                "ok": true,
                "reply": resumed,
            });
        }

        return oxidase::serde_json::json!({
            "ok": false,
            "refused": true,
            "reason": "approval_required",
            "approval_id": approval_id,
            "request": approval_request,
        });
    }

    oxidase::serde_json::json!({
        "ok": true,
        "reply": reply,
    })
}

/// Non-wasm32 stub: pure no-op with zero `dioxus::document::eval` calls and
/// zero `NoOpDocument` noise on native/SSR targets.
#[cfg(not(target_arch = "wasm32"))]
pub fn use_webmcp_bridge(_handle: AgentBridgeHandle, _enabled: bool) {}

/// Arms the browser WebMCP bridge when `enabled` is `true`, registering only
/// `snapshot`, `query`, and `act` against `document?.modelContext ?? navigator?.modelContext`
/// with `{ signal: controller.signal }`, and tearing down cleanly in `use_drop`.
#[cfg(target_arch = "wasm32")]
pub fn use_webmcp_bridge(handle: AgentBridgeHandle, enabled: bool) {
    use dioxus::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum WebMcpInbound {
        Abort {
            abort: bool,
            turn_id: Option<u64>,
            reason: Option<String>,
        },
        ToolCall {
            id: String,
            tool: String,
            #[serde(default)]
            input: oxidase::serde_json::Value,
        },
    }

    #[derive(Serialize)]
    struct WebMcpToolInit {
        name: &'static str,
        description: &'static str,
        read_only_hint: bool,
        untrusted_content_hint: bool,
        consequential_hint: bool,
    }

    #[derive(Serialize)]
    struct WebMcpInitMessage {
        init: bool,
        tools: Vec<WebMcpToolInit>,
    }

    #[derive(Serialize)]
    struct WebMcpReplyMessage {
        id: String,
        result: oxidase::serde_json::Value,
    }

    #[derive(Serialize)]
    struct WebMcpTeardownMessage {
        teardown: bool,
    }

    const WEBMCP_BRIDGE_JS: &str = r#"
        (async () => {
            const mc = (typeof document !== "undefined" && document?.modelContext)
                ?? (typeof navigator !== "undefined" && navigator?.modelContext);
            if (!mc || typeof mc.registerTool !== "function") {
                return;
            }
            const controller = new AbortController();
            const pending = new Map();
            let nextCallId = 1;

            const safeRegister = (tool) => {
                try {
                    const res = mc.registerTool(
                        {
                            name: tool.name,
                            description: tool.description,
                            inputSchema: { type: "object" },
                            annotations: {
                                readOnlyHint: !!tool.read_only_hint,
                                untrustedContentHint: !!tool.untrusted_content_hint,
                                consequentialHint: !!tool.consequential_hint,
                            },
                            execute: async (input, client) => {
                                const id = String(nextCallId++);
                                return await new Promise((resolve, reject) => {
                                    pending.set(id, { resolve, reject });
                                    if (client && client.signal) {
                                        if (client.signal.aborted) {
                                            pending.delete(id);
                                            dioxus.send({ abort: true, reason: "webmcp_signal_aborted" });
                                            reject(new DOMException("Aborted", "AbortError"));
                                            return;
                                        }
                                        client.signal.addEventListener(
                                            "abort",
                                            () => {
                                                pending.delete(id);
                                                dioxus.send({ abort: true, reason: "webmcp_signal_aborted" });
                                                reject(new DOMException("Aborted", "AbortError"));
                                            },
                                            { once: true }
                                        );
                                    }
                                    dioxus.send({ id, tool: tool.name, input: input ?? {} });
                                });
                            },
                        },
                        { signal: controller.signal }
                    );
                    if (res && typeof res.catch === "function") {
                        res.catch((err) => {
                            if (err?.name !== "NotAllowedError" && err?.name !== "AbortError") {
                                console.warn("monoxus webmcp registerTool error:", err);
                            }
                        });
                    }
                } catch (err) {
                    if (err?.name !== "NotAllowedError" && err?.name !== "AbortError") {
                        console.warn("monoxus webmcp registerTool error:", err);
                    }
                }
            };

            while (true) {
                let msg;
                try {
                    msg = await dioxus.recv();
                } catch (_) {
                    break;
                }
                if (!msg || typeof msg !== "object") {
                    continue;
                }
                if (msg.teardown) {
                    controller.abort();
                    for (const [, waiter] of pending) {
                        waiter.reject(new DOMException("WebMCP bridge torn down", "AbortError"));
                    }
                    pending.clear();
                    break;
                }
                if (msg.init && Array.isArray(msg.tools)) {
                    for (const tool of msg.tools) {
                        safeRegister(tool);
                    }
                    continue;
                }
                if (typeof msg.id === "string" && pending.has(msg.id)) {
                    const waiter = pending.get(msg.id);
                    pending.delete(msg.id);
                    waiter.resolve(msg.result);
                }
            }
        })();
    "#;

    let mut active_eval: CopyValue<Option<dioxus::document::Eval>> =
        use_hook(|| CopyValue::new(None));
    let mut handle_slot = use_hook(|| CopyValue::new(handle.clone()));
    if handle_slot.try_read().is_ok_and(|cur| *cur != handle) {
        handle_slot.set(handle);
    }

    use_effect(use_reactive((&enabled,), move |(is_enabled,)| {
        if !is_enabled {
            let prev = active_eval.try_write().ok().and_then(|mut slot| slot.take());
            if let Some(eval_handle) = prev {
                let _ = eval_handle.send(WebMcpTeardownMessage { teardown: true });
            }
            return;
        }

        if active_eval.try_read().is_ok_and(|slot| slot.is_some()) {
            return;
        }

        let eval_handle = dioxus::document::eval(WEBMCP_BRIDGE_JS);
        let tools: Vec<WebMcpToolInit> = webmcp_tool_descriptors()
            .into_iter()
            .map(|desc| WebMcpToolInit {
                name: desc.name,
                description: desc.description,
                read_only_hint: !desc.mutates_ui,
                untrusted_content_hint: true,
                consequential_hint: desc.mutates_ui,
            })
            .collect();
        let _ = eval_handle.send(WebMcpInitMessage { init: true, tools });
        active_eval.set(Some(eval_handle));

        let mut recv_eval = eval_handle;
        spawn(async move {
            while let Ok(msg) = recv_eval.recv::<WebMcpInbound>().await {
                let Ok(bridge) = handle_slot.try_read().map(|g| g.clone()) else {
                    break;
                };
                match msg {
                    WebMcpInbound::Abort {
                        abort: true,
                        turn_id,
                        reason,
                    } => {
                        if let Some(tid) = turn_id {
                            let _ = bridge.handle_request(AgentRequest::AbortTurn {
                                turn_id: tid,
                                reason,
                            });
                        } else {
                            let _ = bridge.abort_active_turn(
                                reason.unwrap_or_else(|| String::from("webmcp signal aborted")),
                            );
                        }
                    }
                    WebMcpInbound::Abort { abort: false, .. } => {}
                    WebMcpInbound::ToolCall { id, tool, input } => {
                        let result = execute_webmcp_call(&bridge, &tool, &input).await;
                        let _ = recv_eval.send(WebMcpReplyMessage { id, result });
                    }
                }
            }
        });
    }));

    use_drop(move || {
        let prev = active_eval.try_write().ok().and_then(|mut slot| slot.take());
        if let Some(eval_handle) = prev {
            let _ = eval_handle.send(WebMcpTeardownMessage { teardown: true });
        }
    });
}

#[cfg(test)]
mod tests {
    use ui_sdk::agent::{ActionCall, AgentRequest, SnapshotValue, TargetQuery};

    use super::{WEBMCP_TOOL_NAMES, translate_webmcp_tool_call, webmcp_tool_descriptors};

    #[test]
    fn webmcp_tool_descriptors_strictly_exclude_resume_approval() {
        let descriptors = webmcp_tool_descriptors();
        let names: Vec<&str> = descriptors.iter().map(|d| d.name).collect();

        assert_eq!(names, WEBMCP_TOOL_NAMES);
        assert!(!names.contains(&"resume_approval"));
        assert_eq!(descriptors.len(), 3);
        assert!(!descriptors[0].mutates_ui); // snapshot
        assert!(!descriptors[1].mutates_ui); // query
        assert!(descriptors[2].mutates_ui); // act
    }

    #[test]
    fn translate_webmcp_tool_call_blocks_resume_approval_and_translates_allowed_tools() {
        let forbidden = translate_webmcp_tool_call(
            "resume_approval",
            &oxidase::serde_json::json!({ "approval_id": 1, "decision": "Allow" }),
        );
        assert!(forbidden.is_err());

        let snap = translate_webmcp_tool_call(
            "snapshot",
            &oxidase::serde_json::json!({ "include_off_page": false }),
        )
        .expect("snapshot translates");
        assert_eq!(
            snap,
            AgentRequest::Snapshot {
                include_off_page: false
            }
        );

        let query = translate_webmcp_tool_call(
            "query",
            &oxidase::serde_json::json!({ "target": "Auto renew" }),
        )
        .expect("query translates");
        assert_eq!(
            query,
            AgentRequest::Query {
                target: TargetQuery::parse("Auto renew")
            }
        );

        let expect = translate_webmcp_tool_call(
            "query",
            &oxidase::serde_json::json!({
                "target": "Auto renew",
                "field": "checked",
                "expected": { "Bool": true }
            }),
        )
        .expect("expect translates");
        assert_eq!(
            expect,
            AgentRequest::Expect {
                target: TargetQuery::parse("Auto renew"),
                field: String::from("checked"),
                expected: SnapshotValue::Bool(true),
            }
        );

        let act = translate_webmcp_tool_call(
            "act",
            &oxidase::serde_json::json!({
                "target": "Auto renew",
                "action": "toggle"
            }),
        )
        .expect("act translates");
        assert_eq!(
            act,
            AgentRequest::Act {
                target: TargetQuery::parse("Auto renew"),
                call: ActionCall::new("toggle"),
            }
        );
    }
}
