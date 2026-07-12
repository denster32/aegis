use crate::config::AegisConfig;
use crate::learn::LearnRuntime;
use crate::prompts;
use aegis_store::Store;
use aegis_tools::{PermissionMode, TodoStore, ToolContext, ToolRegistry};
use aegis_xai::{
    server_tools, system_msg, user_msg, CreateResponseRequest, FunctionCallOutput, InputItem,
    ReasoningConfig, ResponsesClient, TextConfig, TextFormat, ToolChoice, ToolDef, ToolSpec,
};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, info};

/// Print/stream callback for interactive agent output.
pub type PrintFn = Arc<dyn Fn(&str) + Send + Sync>;
/// Interactive ask callback (permissions / ask_user).
pub type AskFn = Arc<dyn Fn(&str) -> String + Send + Sync>;

struct StoreTodoAdapter {
    store: Arc<Store>,
}

impl TodoStore for StoreTodoAdapter {
    fn set_todos(&self, session_id: &str, todos_json: &str) -> anyhow::Result<()> {
        self.store.set_todos(session_id, todos_json)
    }
    fn get_todos(&self, session_id: &str) -> anyhow::Result<Option<String>> {
        self.store.get_todos(session_id)
    }
}

pub struct AgentLoop {
    pub client: ResponsesClient,
    pub store: Arc<Store>,
    pub tools: Arc<ToolRegistry>,
    pub config: AegisConfig,
    pub cwd: PathBuf,
    pub session_id: String,
    pub previous_response_id: Option<String>,
    pub permission: PermissionMode,
    pub print_fn: Option<PrintFn>,
    pub ask_fn: Option<AskFn>,
    /// Override model for this loop (swarm workers).
    pub model_override: Option<String>,
    pub system_override: Option<String>,
    pub bootstrap_context: bool,
    /// Prefer SSE streaming when printing to TTY.
    pub use_streaming: bool,
    /// Project learning / self-heal (optional).
    pub learn: Option<LearnRuntime>,
}

impl AgentLoop {
    pub fn new(
        client: ResponsesClient,
        store: Arc<Store>,
        tools: Arc<ToolRegistry>,
        config: AegisConfig,
        cwd: PathBuf,
        session_id: String,
    ) -> Self {
        Self {
            client,
            store,
            tools,
            config,
            cwd,
            session_id,
            previous_response_id: None,
            permission: PermissionMode::Prompt,
            print_fn: None,
            ask_fn: None,
            model_override: None,
            system_override: None,
            bootstrap_context: true,
            // Streaming is opt-in; non-stream is more reliable for tool loops.
            use_streaming: false,
            learn: None,
        }
    }

    pub fn with_learning(mut self, enabled: bool) -> Self {
        if enabled {
            if let Ok(learn) = LearnRuntime::open(&self.cwd, true) {
                self.learn = Some(learn);
            }
        }
        self
    }

    fn emit(&self, s: &str) {
        if let Some(f) = &self.print_fn {
            f(s);
        } else {
            print!("{s}");
            let _ = std::io::Write::flush(&mut std::io::stdout());
        }
    }

    fn model(&self) -> &str {
        self.model_override
            .as_deref()
            .unwrap_or(self.config.model.as_str())
    }

    fn tool_specs(&self) -> Vec<ToolSpec> {
        let mut specs: Vec<ToolSpec> = self
            .tools
            .to_xai_tools()
            .into_iter()
            .map(|(name, desc, params)| ToolDef::function(name, desc, params).as_spec())
            .collect();
        specs.extend(server_tools(
            self.config.web_search,
            self.config.x_search,
            self.config.code_execution,
        ));
        specs
    }

    /// Whether this model accepts Responses `reasoning.effort` (grok-4.5 family).
    /// Worker models like `grok-code-fast-1` return 400 if reasoning is sent.
    pub fn model_supports_reasoning(model: &str) -> bool {
        crate::model_supports_reasoning(model)
    }

    fn reasoning_for_step(&self, step: usize, had_tools_last: bool) -> Option<ReasoningConfig> {
        if !Self::model_supports_reasoning(self.model()) {
            return None;
        }
        // First planning-ish step may need more thought; pure tool loops prefer low.
        let effort = if step <= 1 && !had_tools_last {
            self.config.reasoning_effort.as_str()
        } else {
            self.config.tool_reasoning_effort.as_str()
        };
        Some(ReasoningConfig::parse_effort(effort))
    }

    fn tool_context(&self) -> ToolContext {
        let mut ctx = ToolContext::new(self.cwd.clone(), self.session_id.clone(), self.permission);
        ctx.ask = self.ask_fn.clone();
        ctx.todo_store = Some(Arc::new(StoreTodoAdapter {
            store: self.store.clone(),
        }));
        ctx
    }

    /// Run one user turn to completion (tool loop until no more calls).
    pub async fn run_turn(&mut self, user_text: &str) -> Result<String> {
        self.store
            .append_message(&self.session_id, "user", user_text)?;
        if let Some(learn) = self.learn.as_mut() {
            learn.note(format!("USER: {user_text}"));
        }

        let mut input: Vec<InputItem> = Vec::new();

        // First turn of session: system + bootstrap
        if self.previous_response_id.is_none() {
            let mut sys = self
                .system_override
                .clone()
                .unwrap_or_else(|| prompts::system_prompt(&self.cwd.display().to_string()));
            if self.learn.as_ref().map(|l| l.enabled).unwrap_or(false) {
                sys.push_str(
                    "\n\nLearning: Prefer project memory lessons and known fixes. \
                     After verified wins, note conventions. On tool failures, self-heal before giving up. \
                     You may use memory_read/memory_write tools when available.",
                );
            }
            input.push(system_msg(sys));
            if self.bootstrap_context {
                let include_mem = self.learn.as_ref().map(|l| l.enabled).unwrap_or(true);
                let pack = aegis_context::pack_workspace_with_memory(&self.cwd, include_mem);
                input.push(user_msg(format!(
                    "[workspace context — reference only]\n{pack}"
                )));
            }
        }

        input.push(user_msg(user_text.to_string()));

        let mut final_text = String::new();
        let mut steps = 0usize;
        // After a tool-bearing step, lower reasoning for subsequent tool-loop turns.
        let mut had_tools_last = false;
        // Credit heal when a later tool step succeeds after self-heal guidance.
        let mut pending_heal_credit = false;
        let mut heal_credited = false;

        loop {
            steps += 1;
            if steps > self.config.max_agent_steps {
                anyhow::bail!("max agent steps ({}) exceeded", self.config.max_agent_steps);
            }

            let reasoning = self.reasoning_for_step(steps, had_tools_last);
            let req = CreateResponseRequest {
                model: self.model().to_string(),
                input: input.clone(),
                tools: Some(self.tool_specs()),
                tool_choice: Some(ToolChoice::auto()),
                previous_response_id: self.previous_response_id.clone(),
                // Tool loops require server-side store so previous_response_id chains.
                store: Some(true),
                stream: Some(false),
                temperature: None,
                max_output_tokens: None,
                parallel_tool_calls: Some(true),
                text: None,
                include: None,
                reasoning: reasoning.clone(),
                prompt_cache_key: Some(self.session_id.clone()),
            };

            debug!(
                step = steps,
                model = %self.model(),
                reasoning = reasoning.as_ref().map(|r| r.effort.as_str()).unwrap_or("none"),
                cache_key = %self.session_id,
                "agent step"
            );
            let allow_stream = self.use_streaming && steps == 1;
            let resp = if allow_stream {
                let emit = self.print_fn.clone();
                self.client
                    .create_stream_with_callback(req, move |ev| {
                        if let aegis_xai::StreamEvent::TextDelta(d) = ev {
                            if let Some(f) = &emit {
                                f(&d);
                            } else {
                                print!("{d}");
                                let _ = std::io::Write::flush(&mut std::io::stdout());
                            }
                        }
                    })
                    .await
                    .context("xAI responses.create (stream)")?
            } else {
                self.client
                    .create(req)
                    .await
                    .context("xAI responses.create")?
            };

            if let Some(u) = &resp.usage {
                let _ = self.store.add_usage(
                    &self.session_id,
                    u.input_tokens.unwrap_or(0),
                    u.output_tokens.unwrap_or(0),
                );
                let cached = u.cached_tokens();
                let reasoning_toks = u.reasoning_token_count();
                if cached > 0 || reasoning_toks > 0 {
                    debug!(cached, reasoning_toks, "xAI usage details");
                }
            }

            self.previous_response_id = Some(resp.id.clone());
            let _ = self
                .store
                .set_previous_response_id(&self.session_id, Some(&resp.id));

            let text = resp.output_text();
            if !text.is_empty() {
                let has_tools = !resp.function_calls().is_empty();
                // Non-stream: always print. Stream chat (no tools): deltas already printed.
                // Stream+tools: print nothing extra (tools follow).
                if !allow_stream {
                    self.emit(&text);
                    if !text.ends_with('\n') {
                        self.emit("\n");
                    }
                } else if !has_tools {
                    // Ensure trailing newline after streamed chat
                    if !text.ends_with('\n') {
                        self.emit("\n");
                    }
                }
                final_text = text;
            }

            // Server-side tools (web_search, code_execution, …) complete on xAI —
            // only client function_calls need local exec.
            let calls = resp.function_calls();
            if calls.is_empty() {
                if !final_text.is_empty() {
                    self.store
                        .append_message(&self.session_id, "assistant", &final_text)?;
                }
                break;
            }
            had_tools_last = true;

            // Execute tools in parallel
            let sem = Arc::new(Semaphore::new(self.config.max_tool_parallel));
            let ctx = Arc::new(self.tool_context());
            let tools = self.tools.clone();
            let mut join_set = tokio::task::JoinSet::new();

            for call in calls {
                let call_id = call.call_id.to_string();
                let name = call.name.to_string();
                let args_str = call.arguments.to_string();
                let sem = sem.clone();
                let ctx = ctx.clone();
                let tools = tools.clone();
                let emit = self.print_fn.clone();

                let cwd_for_hooks = self.cwd.clone();
                join_set.spawn(async move {
                    let _permit = sem.acquire().await.ok();
                    let msg = format!("{}\n", crate::ui::tool_call(&name));
                    if let Some(f) = &emit {
                        f(&msg);
                    } else {
                        eprint!("{msg}");
                    }

                    let _ = crate::hooks::run_hook(
                        &cwd_for_hooks,
                        "pre-tool",
                        &[
                            ("AEGIS_TOOL", name.as_str()),
                            ("AEGIS_TOOL_ARGS", args_str.as_str()),
                        ],
                    );

                    let args: serde_json::Value = serde_json::from_str(&args_str)
                        .unwrap_or_else(|_| serde_json::json!({ "_raw": args_str }));

                    let result = match tools.get(&name) {
                        Some(tool) => tool.call(args, &ctx).await,
                        None => aegis_tools::ToolResult::err(format!("unknown tool: {name}")),
                    };

                    let ok = result.ok;
                    let out = if ok {
                        result.output
                    } else {
                        format!("ERROR: {}", result.output)
                    };
                    let truncated = if out.chars().count() > 80_000 {
                        format!("{}\n[truncated]", crate::utf8_truncate(&out, 80_000))
                    } else {
                        out
                    };
                    let status = if ok { "ok" } else { "err" };
                    let _ = crate::hooks::run_hook(
                        &cwd_for_hooks,
                        "post-tool",
                        &[("AEGIS_TOOL", name.as_str()), ("AEGIS_TOOL_STATUS", status)],
                    );
                    (call_id, name, truncated, result.ok)
                });
            }

            // Next request: only tool outputs + previous_response_id (stateful API)
            input.clear();
            let mut heal_notes: Vec<String> = Vec::new();
            let mut any_ok = false;
            let mut _any_err = false;
            while let Some(joined) = join_set.join_next().await {
                let (call_id, name, output, ok) = joined.context("tool join")?;
                info!(%name, len = output.len(), ok, "tool done");
                let is_err = !ok || output.starts_with("ERROR:");
                if is_err {
                    _any_err = true;
                } else {
                    any_ok = true;
                }
                if let Some(learn) = self.learn.as_mut() {
                    learn.note(format!(
                        "TOOL {name}: {}",
                        crate::utf8_truncate(&output, 300)
                    ));
                    if is_err {
                        if let Some(heal) = learn.on_tool_error(&name, &output) {
                            heal_notes.push(heal);
                        }
                    }
                }
                let preview = crate::utf8_truncate(&output, 200);
                self.emit(&format!("{}\n", crate::ui::tool_done(&name)));
                debug!(%preview, "tool output preview");
                input.push(InputItem::FunctionCallOutput(FunctionCallOutput::new(
                    call_id, output,
                )));
            }
            // Prior heal guidance + any subsequent tool success → credit once.
            // Require at least one ok tool after guidance; do not require a fully clean
            // parallel batch (mixed ok/err is common while recovering compile failures).
            if pending_heal_credit && any_ok && !heal_credited {
                if let Some(learn) = self.learn.as_mut() {
                    learn.record_successful_heal(
                        "session",
                        "tool error recovered after self-heal guidance",
                        "subsequent tool(s) succeeded after heal guidance",
                    );
                }
                heal_credited = true;
                pending_heal_credit = false;
            }
            // Inject self-heal guidance as a synthetic user note for next model step
            if !heal_notes.is_empty() {
                pending_heal_credit = true;
                let combined = heal_notes.join("\n\n");
                input.push(user_msg(format!(
                    "[system self-heal guidance — follow this next]\n{combined}"
                )));
            }
        }

        if let Some(learn) = self.learn.as_mut() {
            if !final_text.is_empty() {
                learn.note(format!("ASSISTANT: {}", truncate_str(&final_text, 500)));
            }
        }

        Ok(final_text)
    }

    /// Run end-of-session reflection into project memory.
    pub async fn reflect_and_save(&mut self) -> Result<()> {
        if let Some(learn) = self.learn.as_mut() {
            let model = self.config.model.clone();
            let client = self.client.clone();
            let r = learn.reflect(&client, &model).await?;
            if !r.wins.is_empty() || !r.new_lessons.is_empty() {
                self.emit(&format!(
                    "{}\n",
                    crate::ui::event(
                        "learn",
                        format!(
                            "{} lesson(s) · {} win(s)",
                            r.new_lessons.len(),
                            r.wins.len()
                        )
                    )
                ));
            }
            if let Some(sug) = r.agents_md_suggestion {
                if !sug.is_empty() {
                    self.emit(&format!(
                        "{}\n{}\n",
                        crate::ui::label("agents.md"),
                        crate::ui::note(sug)
                    ));
                }
            }
        }
        Ok(())
    }

    /// Structured JSON completion (no tools) for plans / DAGs.
    pub async fn structured_json(
        &mut self,
        system: &str,
        user: &str,
        schema_name: &str,
        schema: serde_json::Value,
    ) -> Result<String> {
        let mut input = vec![system_msg(system), user_msg(user)];
        if self.bootstrap_context && self.previous_response_id.is_none() {
            let pack = aegis_context::pack_workspace(&self.cwd);
            input.insert(1, user_msg(format!("[workspace context]\n{pack}")));
        }

        // Cache structured pure-LLM calls
        let cache_key =
            Store::cache_key(&[self.model(), system, user, schema_name, &schema.to_string()]);
        if let Ok(Some(hit)) = self.store.cache_get(&cache_key) {
            debug!("structured_json cache hit");
            return Ok(hit);
        }

        let reasoning = if Self::model_supports_reasoning(self.model()) {
            Some(aegis_xai::ReasoningConfig::high())
        } else {
            None
        };
        let req = CreateResponseRequest {
            model: self.model().to_string(),
            input,
            tools: None,
            tool_choice: None, // must omit when no tools (xAI 400 otherwise)
            previous_response_id: None, // independent structured call
            store: Some(false),
            stream: Some(false),
            temperature: Some(0.2),
            max_output_tokens: Some(8192),
            parallel_tool_calls: None,
            text: Some(TextConfig {
                format: TextFormat::JsonSchema {
                    name: schema_name.into(),
                    schema,
                    strict: Some(true),
                },
            }),
            include: None,
            reasoning: reasoning.clone(),
            prompt_cache_key: Some(format!("aegis-{}", std::process::id())),
        };

        let resp = match self.client.create(req).await {
            Ok(r) => r,
            Err(e) => {
                // Fallback: ask for JSON in prompt without schema enforcement
                tracing::warn!(error = %e, "structured schema mode failed; plain JSON fallback");
                let fallback = CreateResponseRequest {
                    model: self.model().to_string(),
                    input: vec![
                        system_msg(format!(
                            "{system}\nRespond with ONLY valid JSON matching the requested schema. No markdown fences."
                        )),
                        user_msg(format!("{user}\n\nSchema name: {schema_name}")),
                    ],
                    tools: None,
                    tool_choice: None,
                    previous_response_id: None,
                    store: Some(false),
                    stream: Some(false),
                    temperature: Some(0.2),
                    max_output_tokens: Some(8192),
                    parallel_tool_calls: None,
                    text: None,
                    include: None,
                    reasoning,
                    prompt_cache_key: Some(format!("aegis-{}", std::process::id())),
                };
                self.client.create(fallback).await?
            }
        };
        if let Some(u) = &resp.usage {
            let _ = self.store.add_usage(
                &self.session_id,
                u.input_tokens.unwrap_or(0),
                u.output_tokens.unwrap_or(0),
            );
        }
        let text = resp.output_text();
        let text = extract_json_object(&text).unwrap_or(text);
        let _ = self.store.cache_put(&cache_key, &text);
        Ok(text)
    }
}

fn extract_json_object(text: &str) -> Option<String> {
    let t = text.trim();
    if t.starts_with('{') {
        return Some(t.to_string());
    }
    if let Some(start) = t.find('{') {
        if let Some(end) = t.rfind('}') {
            if end > start {
                return Some(t[start..=end].to_string());
            }
        }
    }
    None
}

fn truncate_str(s: &str, n: usize) -> String {
    crate::utf8_truncate(s, n)
}
