use aegis_auth::{
    auth_paths, clear_auth_file, device_login, import_grok_to_aegis, AuthProvider, TokenSource,
};
use aegis_core::{
    assess_v2, automations, checkpoint_create, checkpoint_list, checkpoint_restore, factory_status,
    format_factory, format_readiness_v2, generate_wiki, install_dream_cron, install_qa,
    install_review_workflow, install_wiki_workflow, missions_new, missions_run, missions_status,
    readiness_report, review_diff, review_pr, run_dream, run_mission, run_plan, run_qa,
    AegisConfig, AgentLoop, DreamOptions, Effort, MissionOptions,
};
use aegis_memory::ProjectMemory;
use aegis_store::{AegisPaths, Store};
use aegis_tools::{default_registry, PermissionMode, ToolRegistry};
use aegis_xai::{ResponsesClient, TokenSource as XaiTokenSource};
use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::{Parser, Subcommand};
use console::style;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

/// Bridge aegis-auth → aegis-xai token traits.
struct AuthBridge(Arc<AuthProvider>);

#[async_trait]
impl XaiTokenSource for AuthBridge {
    async fn bearer_token(&self) -> anyhow::Result<String> {
        TokenSource::token(self.0.as_ref()).await
    }
    async fn on_unauthorized(&self) -> anyhow::Result<()> {
        TokenSource::force_refresh(self.0.as_ref()).await
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "aegis",
    version,
    about = "Sovereign Grok 4.5 coding agent (Rust)"
)]
struct Cli {
    /// One-shot prompt (non-interactive)
    #[arg(short = 'p', long = "print", global = true)]
    prompt: Option<String>,

    /// Model override (default: grok-4.5)
    #[arg(short, long, global = true)]
    model: Option<String>,

    /// Reasoning/cost effort: low | medium | high
    #[arg(long, default_value = "medium", global = true)]
    effort: String,

    /// Auto-approve all tools
    #[arg(long, global = true)]
    yolo: bool,

    /// Working directory
    #[arg(long, global = true)]
    cwd: Option<PathBuf>,

    /// Resume session id
    #[arg(long, global = true)]
    session: Option<String>,

    /// Enable SSE streaming (experimental)
    #[arg(long, global = true)]
    stream: bool,

    /// Disable project learning (no self-heal / reflect / memory inject)
    #[arg(long, global = true)]
    no_learn: bool,

    /// Verbose logs
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a multi-agent mission (DAG swarm)
    Mission {
        goal: String,
        #[arg(long, default_value_t = 4)]
        workers: usize,
        #[arg(long)]
        approve: bool,
    },
    /// Structured plan only
    Plan { goal: String },
    /// Factory-style Missions (plan → Mission Control → execute)
    Missions {
        #[command(subcommand)]
        action: MissionsCmd,
    },
    /// Project readiness checklist (use --v2 for L1–L5 pillars)
    Readiness {
        #[arg(long)]
        json: bool,
        #[arg(long, default_value_t = true)]
        v2: bool,
    },
    /// Nightly dream — deep self-improve / reflect
    Dream {
        #[arg(long)]
        apply: bool,
        #[arg(long, default_value = "high")]
        budget: String,
        #[command(subcommand)]
        action: Option<DreamCmd>,
    },
    /// Software Factory SDLC coverage map
    Factory,
    /// Project wiki generate / refresh
    Wiki {
        #[command(subcommand)]
        action: WikiCmd,
    },
    /// Code review (PR or local diff)
    Review {
        #[arg(long)]
        pr: Option<u64>,
        #[arg(long)]
        diff: bool,
        #[arg(long, default_value = "deep")]
        depth: String,
    },
    /// Install Automated QA skills
    InstallQa,
    /// Run Automated QA
    Qa {
        #[arg(long)]
        base: Option<String>,
    },
    /// Install GH code-review workflow
    InstallCodeReview,
    /// Install GH wiki-refresh workflow
    InstallWikiRefresh,
    /// Git checkpoint / restore
    Checkpoint {
        #[command(subcommand)]
        action: CheckpointCmd,
    },
    /// Vision: describe an image
    Vision {
        path: String,
        #[arg(long, default_value = "Describe this image and note any issues.")]
        question: String,
    },
    /// File-based automations
    Automation {
        #[command(subcommand)]
        action: AutomationCmd,
    },
    /// Project memory (learning)
    Memory {
        #[command(subcommand)]
        action: MemoryCmd,
    },
    /// Session management
    Session {
        #[command(subcommand)]
        action: SessionCmd,
    },
    /// Authenticate with Grok OAuth
    Login {
        /// Use device-code flow (default on headless)
        #[arg(long, default_value_t = true)]
        device: bool,
        /// Import existing ~/.grok/auth.json into Aegis
        #[arg(long)]
        import_grok: bool,
        /// Write tokens to ~/.grok/auth.json instead of Aegis data dir
        #[arg(long)]
        write_grok: bool,
    },
    /// Clear Aegis-stored credentials (does not wipe Grok CLI auth)
    Logout,
    /// Auth / credential helpers
    Auth {
        #[command(subcommand)]
        action: AuthCmd,
    },
}

#[derive(Subcommand, Debug)]
enum AuthCmd {
    /// Show current credential source (no secrets)
    Status,
    /// Force OIDC token refresh
    Refresh,
}

#[derive(Subcommand, Debug)]
enum SessionCmd {
    List {
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    Show {
        id: String,
    },
}

#[derive(Subcommand, Debug)]
enum MemoryCmd {
    /// Show project memory summary
    Show,
    /// Clear lessons and failures (keeps MEMORY.md)
    Clear,
    /// Print MEMORY.md
    Dump,
}

#[derive(Subcommand, Debug)]
enum MissionsCmd {
    /// Create a mission plan (one-shot structured)
    New {
        goal: String,
        #[arg(long, default_value_t = true)]
        oneshot: bool,
    },
    /// List missions
    List,
    /// Mission Control board
    Status { id: Option<String> },
    /// Execute an approved mission
    Run { id: String },
}

#[derive(Subcommand, Debug)]
enum DreamCmd {
    /// Install nightly crontab + automation file
    Install,
}

#[derive(Subcommand, Debug)]
enum WikiCmd {
    Generate,
    Refresh,
}

#[derive(Subcommand, Debug)]
enum CheckpointCmd {
    Create {
        #[arg(default_value = "manual")]
        label: String,
    },
    List,
    Restore {
        id: String,
    },
}

#[derive(Subcommand, Debug)]
enum AutomationCmd {
    List,
    Ensure,
    Run { name: String },
    InstallAll,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    let paths = AegisPaths::default_dirs()?;
    paths.ensure()?;
    let store = Arc::new(Store::open(&paths)?);

    let cwd_early = cli
        .cwd
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let cwd_early = cwd_early.canonicalize().unwrap_or(cwd_early);

    // Commands that need no network auth
    match &cli.command {
        Some(Commands::Memory { action }) => {
            let mem = ProjectMemory::open(&cwd_early)?;
            match action {
                MemoryCmd::Show => println!("{}", mem.summary_report()?),
                MemoryCmd::Clear => {
                    mem.clear_lessons_failures()?;
                    println!("cleared LESSONS.jsonl and FAILURES.jsonl");
                }
                MemoryCmd::Dump => println!("{}", mem.read_memory_md()?),
            }
            return Ok(());
        }
        Some(Commands::Readiness { json, v2 }) => {
            if *v2 {
                let r = assess_v2(&cwd_early);
                if *json {
                    println!("{}", serde_json::to_string_pretty(&r)?);
                } else {
                    print!("{}", format_readiness_v2(&r));
                }
            } else {
                print!("{}", readiness_report(&cwd_early));
            }
            return Ok(());
        }
        Some(Commands::Factory) => {
            print!("{}", format_factory(&factory_status(&cwd_early)));
            return Ok(());
        }
        Some(Commands::Automation {
            action: AutomationCmd::List,
        }) => {
            let _ = automations::ensure_defaults(&cwd_early);
            print!(
                "{}",
                automations::format_list(&automations::list(&cwd_early)?)
            );
            return Ok(());
        }
        Some(Commands::Automation {
            action: AutomationCmd::Ensure,
        }) => {
            automations::ensure_defaults(&cwd_early)?;
            println!("default automations ensured under .aegis/automations/");
            return Ok(());
        }
        Some(Commands::Missions {
            action: MissionsCmd::List,
        }) => {
            print!("{}", missions_status(&cwd_early, None)?);
            return Ok(());
        }
        Some(Commands::Missions {
            action: MissionsCmd::Status { id },
        }) => {
            print!("{}", missions_status(&cwd_early, id.as_deref())?);
            return Ok(());
        }
        Some(Commands::Session {
            action: SessionCmd::List { limit },
        }) => {
            for s in store.list_sessions(*limit)? {
                println!(
                    "{}  {}  {}  in={} out={}",
                    style(&s.id[..8.min(s.id.len())]).cyan(),
                    s.updated_at,
                    s.model,
                    s.total_input_tokens,
                    s.total_output_tokens
                );
                println!("    {}", s.cwd);
            }
            return Ok(());
        }
        Some(Commands::Session {
            action: SessionCmd::Show { id },
        }) => {
            let s = resolve_session(&store, id)?;
            println!(
                "id={} model={} cwd={}\nin_tokens={} out_tokens={}\nprev_response={:?}",
                s.id,
                s.model,
                s.cwd,
                s.total_input_tokens,
                s.total_output_tokens,
                s.previous_response_id
            );
            for m in store.messages(&s.id)? {
                println!(
                    "[{}] {}: {}",
                    m.created_at,
                    m.role,
                    truncate(&m.content, 240)
                );
            }
            return Ok(());
        }
        Some(Commands::Login {
            import_grok,
            write_grok,
            ..
        }) => {
            if *import_grok {
                let e = import_grok_to_aegis()?;
                println!(
                    "Imported Grok OAuth into Aegis (email={:?}, expires={:?})",
                    e.email, e.expires_at
                );
                return Ok(());
            }
            let ap = auth_paths();
            let entry = device_login(&ap, !*write_grok).await?;
            println!(
                "Login OK. source={} email={:?} expires={:?}",
                if *write_grok { "grok" } else { "aegis" },
                entry.email,
                entry.expires_at
            );
            return Ok(());
        }
        Some(Commands::Logout) => {
            let ap = auth_paths();
            clear_auth_file(&ap.aegis)?;
            println!("Cleared Aegis auth file ({})", ap.aegis.display());
            println!("Note: ~/.grok/auth.json left intact. Run `grok logout` to clear that.");
            return Ok(());
        }
        Some(Commands::Auth {
            action: AuthCmd::Status,
        }) => {
            match AuthProvider::resolve() {
                Ok(p) => {
                    let st = p.status();
                    println!("source:        {}", st.source.as_str());
                    println!("email:         {}", st.email.as_deref().unwrap_or("-"));
                    println!("expires_at:    {}", st.expires_at.as_deref().unwrap_or("-"));
                    println!("team_id:       {}", st.team_id.as_deref().unwrap_or("-"));
                    println!("auth_mode:     {}", st.auth_mode.as_deref().unwrap_or("-"));
                    println!("path:          {}", st.path.as_deref().unwrap_or("-"));
                    println!("needs_refresh: {}", st.needs_refresh);
                }
                Err(e) => {
                    println!("not signed in: {e:#}");
                    std::process::exit(1);
                }
            }
            return Ok(());
        }
        Some(Commands::Auth {
            action: AuthCmd::Refresh,
        }) => {
            let p = AuthProvider::resolve()?;
            p.force_refresh().await?;
            let st = p.status();
            println!(
                "refreshed. expires_at={}",
                st.expires_at.as_deref().unwrap_or("-")
            );
            return Ok(());
        }
        _ => {}
    }

    let auth = AuthProvider::resolve().context("resolve credentials")?;
    let st = auth.status();
    if cli.verbose {
        eprintln!(
            "auth: {} email={:?} expires={:?}",
            st.source.as_str(),
            st.email,
            st.expires_at
        );
    }

    let project_cfg = cli
        .cwd
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join(".aegis/config.toml");
    let project_cfg = if project_cfg.exists() {
        Some(project_cfg)
    } else {
        None
    };

    let effort = Effort::parse(&cli.effort);
    let mut config =
        AegisConfig::load_layered(Some(&paths.config), project_cfg.as_ref()).with_effort(effort);
    if let Some(m) = &cli.model {
        config.model = m.clone();
    }
    if cli.yolo {
        config.yolo = true;
    }

    let cwd = cli
        .cwd
        .clone()
        .unwrap_or_else(|| std::env::current_dir().context("cwd").unwrap());
    let cwd = cwd.canonicalize().unwrap_or(cwd);

    let tokens: Arc<dyn XaiTokenSource> = Arc::new(AuthBridge(auth.clone()));
    let client = ResponsesClient::new(tokens)?.with_base_url(&config.base_url);

    let mut registry = default_registry();
    // Optional MCP servers from config
    if !config.mcp_servers.is_empty() {
        let servers: Vec<aegis_mcp::McpServerConfig> = config
            .mcp_servers
            .iter()
            .map(|s| aegis_mcp::McpServerConfig {
                name: s.name.clone(),
                command: s.command.clone(),
                args: s.args.clone(),
                env: vec![],
            })
            .collect();
        if let Err(e) = aegis_mcp::register_mcp_tools(&mut registry, &servers).await {
            eprintln!("{} MCP: {e:#}", style("warn").yellow());
        }
    }
    let tools = Arc::new(registry);

    let (session_id, prev) = if let Some(id) = &cli.session {
        let s = resolve_session(&store, id)?;
        (s.id, s.previous_response_id)
    } else {
        let s = store.create_session(&cwd, &config.model)?;
        (s.id, None)
    };

    let mut agent = AgentLoop::new(
        client,
        store.clone(),
        tools,
        config.clone(),
        cwd.clone(),
        session_id.clone(),
    );
    agent.previous_response_id = prev;
    agent.use_streaming = cli.stream;
    if !cli.no_learn {
        agent = agent.with_learning(true);
    }
    // YOLO for explicit flag, one-shot -p, or non-interactive mission/plan commands.
    let auto_yolo = config.yolo
        || cli.prompt.is_some()
        || matches!(
            cli.command,
            Some(Commands::Mission { .. })
                | Some(Commands::Plan { .. })
                | Some(Commands::Missions { .. })
        );
    agent.permission = if auto_yolo {
        PermissionMode::Yolo
    } else {
        PermissionMode::Prompt
    };

    let ask: Arc<dyn Fn(&str) -> String + Send + Sync> = Arc::new(|prompt: &str| {
        eprint!("{prompt}");
        let _ = io::stderr().flush();
        let mut line = String::new();
        let _ = io::stdin().lock().read_line(&mut line);
        line
    });
    agent.ask_fn = Some(ask.clone());

    match cli.command {
        Some(Commands::Mission {
            goal,
            workers,
            approve,
        }) => {
            let opts = MissionOptions {
                auto_approve_graph: !approve,
                max_validate_retries: 1,
                workers,
            };
            let out = run_mission(agent, &goal, opts).await?;
            println!("\n{}", style("── done ──").dim());
            println!("{out}");
        }
        Some(Commands::Plan { goal }) => {
            let plan = run_plan(&mut agent, &goal).await?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
            let _ = agent.reflect_and_save().await;
        }
        Some(Commands::Missions {
            action: MissionsCmd::New { goal, oneshot },
        }) => {
            let plan = missions_new(&mut agent, &goal, oneshot).await?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
            println!(
                "\n{} Mission {} saved. Run: aegis missions run {}",
                style("◆").magenta(),
                plan.id,
                &plan.id[..8]
            );
        }
        Some(Commands::Missions {
            action: MissionsCmd::Run { id },
        }) => {
            let out = missions_run(agent, &id).await?;
            println!("{out}");
        }
        Some(Commands::Dream {
            action: Some(DreamCmd::Install),
            ..
        }) => {
            let msg = install_dream_cron(&cwd, 3)?;
            println!("{msg}");
        }
        Some(Commands::Dream {
            apply,
            budget,
            action: None,
        }) => {
            let model = if budget == "low" {
                "grok-4-fast".to_string()
            } else {
                config.model.clone()
            };
            let opts = DreamOptions {
                apply_memory: true,
                apply_code: apply,
                budget_model: model,
                max_proposals: 5,
                refresh_wiki: true,
            };
            println!("{} starting dream…", style("◆").magenta());
            let journal = run_dream(&agent.client, &cwd, opts).await?;
            println!(
                "Dream {} complete. Applied: {:?}\nProposals: {}",
                journal.id,
                journal.applied,
                journal.proposals.len()
            );
            for p in &journal.proposals {
                println!("  • [{}] {}", p.kind, p.title);
            }
            println!("Journal under .aegis/dreams/");
        }
        Some(Commands::Wiki { action }) => {
            let model = config.model.clone();
            match action {
                WikiCmd::Generate | WikiCmd::Refresh => {
                    let n = generate_wiki(&cwd, &agent.client, &model).await?;
                    println!("Wrote {n} wiki pages under docs/wiki/");
                }
            }
        }
        Some(Commands::Review { pr, diff, depth }) => {
            let model = config.model.clone();
            let report = if let Some(n) = pr {
                review_pr(&agent.client, &model, &cwd, n, &depth).await?
            } else if diff {
                review_diff(&agent.client, &model, &cwd, &depth).await?
            } else {
                anyhow::bail!("pass --pr N or --diff");
            };
            println!("{}\napprove={}", report.summary, report.approve);
            for f in &report.findings {
                println!("  [{}] {} — {}", f.severity, f.title, f.detail);
            }
        }
        Some(Commands::Checkpoint { action }) => match action {
            CheckpointCmd::Create { label } => {
                let cp = checkpoint_create(&cwd, &label)?;
                println!("checkpoint {} (stash={:?})", cp.id, cp.stash_ref);
            }
            CheckpointCmd::List => {
                for c in checkpoint_list(&cwd)? {
                    println!("{}  {}  {}", c.id, c.label, c.created_at);
                }
            }
            CheckpointCmd::Restore { id } => {
                println!("{}", checkpoint_restore(&cwd, &id)?);
            }
        },
        Some(Commands::Vision { path, question }) => {
            let p = std::path::PathBuf::from(&path);
            let p = if p.is_absolute() { p } else { cwd.join(p) };
            let out = aegis_tools::describe_image_file(&p, &question).await?;
            println!("{out}");
        }
        Some(Commands::InstallQa) => {
            println!("{}", install_qa(&cwd)?);
        }
        Some(Commands::Qa { base }) => {
            println!("{}", run_qa(&cwd, base.as_deref())?);
        }
        Some(Commands::InstallCodeReview) => {
            let p = install_review_workflow(&cwd)?;
            println!("wrote {}", p.display());
        }
        Some(Commands::InstallWikiRefresh) => {
            let p = install_wiki_workflow(&cwd)?;
            println!("wrote {}", p.display());
        }
        Some(Commands::Automation {
            action: AutomationCmd::Run { name },
        }) => {
            println!("{}", automations::run(&cwd, &name)?);
        }
        Some(Commands::Automation {
            action: AutomationCmd::InstallAll,
        }) => {
            automations::ensure_defaults(&cwd)?;
            let _ = install_dream_cron(&cwd, 3)?;
            let _ = install_review_workflow(&cwd);
            let _ = install_wiki_workflow(&cwd);
            let _ = install_qa(&cwd);
            println!("installed automations + default workflows/skills");
        }
        Some(Commands::Missions { .. })
        | Some(Commands::Memory { .. })
        | Some(Commands::Readiness { .. })
        | Some(Commands::Factory)
        | Some(Commands::Automation { .. })
        | Some(Commands::Session { .. })
        | Some(Commands::Login { .. })
        | Some(Commands::Logout)
        | Some(Commands::Auth { .. }) => unreachable!(),
        None => {
            if let Some(p) = cli.prompt {
                let _ = agent.run_turn(&p).await?;
                let _ = agent.reflect_and_save().await;
            } else {
                repl(agent, store).await?;
            }
        }
    }

    Ok(())
}

fn resolve_session(store: &Store, id: &str) -> Result<aegis_store::SessionMeta> {
    store
        .get_session(id)?
        .or_else(|| {
            store
                .list_sessions(200)
                .ok()
                .and_then(|list| list.into_iter().find(|s| s.id.starts_with(id)))
        })
        .context("session not found")
}

async fn repl(mut agent: AgentLoop, store: Arc<Store>) -> Result<()> {
    let st_auth = ""; // decorative
    let _ = st_auth;
    println!(
        "{} {}  session {}  model {}",
        style("aegis").bold().cyan(),
        env!("CARGO_PKG_VERSION"),
        style(&agent.session_id[..8]).dim(),
        style(&agent.config.model).green()
    );
    println!(
        "cwd {}  |  /quit /plan /mission /missions /memory /yolo /cost /compact /model /clear\n",
        agent.cwd.display()
    );

    let stdin = io::stdin();
    loop {
        eprint!("{} ", style("❯").cyan().bold());
        let _ = io::stderr().flush();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line == "/quit" || line == "/exit" {
            let _ = agent.reflect_and_save().await;
            break;
        }
        if line == "/memory" {
            if let Ok(mem) = ProjectMemory::open(&agent.cwd) {
                println!("{}", mem.summary_report().unwrap_or_default());
            }
            continue;
        }
        if line.starts_with("/missions") {
            let rest = line.strip_prefix("/missions").unwrap_or("").trim();
            if rest.is_empty() || rest == "list" {
                println!("{}", missions_status(&agent.cwd, None).unwrap_or_default());
            } else if let Some(id) = rest.strip_prefix("status") {
                println!(
                    "{}",
                    missions_status(&agent.cwd, Some(id.trim())).unwrap_or_default()
                );
            } else if let Some(goal) = rest.strip_prefix("new ") {
                match missions_new(&mut agent, goal.trim(), true).await {
                    Ok(p) => println!(
                        "created mission {} — aegis missions run {}",
                        p.id,
                        &p.id[..8]
                    ),
                    Err(e) => eprintln!("missions new: {e:#}"),
                }
            } else {
                eprintln!("usage: /missions [list|status <id>|new <goal>]");
            }
            continue;
        }
        if line == "/yolo" {
            agent.permission = PermissionMode::Yolo;
            agent.config.yolo = true;
            println!("yolo on");
            continue;
        }
        if line == "/cost" {
            if let Ok(Some(s)) = store.get_session(&agent.session_id) {
                println!(
                    "session {}  in={}  out={}  total={}",
                    &s.id[..8],
                    s.total_input_tokens,
                    s.total_output_tokens,
                    s.total_input_tokens + s.total_output_tokens
                );
            }
            continue;
        }
        if line == "/compact" {
            match agent
                .run_turn(
                    "Summarize our conversation so far into a compact bullet list for re-anchoring. Focus on goals, decisions, files touched, and open tasks.",
                )
                .await
            {
                Ok(_) => println!("(compacted)"),
                Err(e) => eprintln!("compact error: {e:#}"),
            }
            continue;
        }
        if line == "/clear" {
            agent.previous_response_id = None;
            let _ = store.set_previous_response_id(&agent.session_id, None);
            println!("cleared server-side chain (local session id kept)");
            continue;
        }
        if let Some(m) = line.strip_prefix("/model ") {
            agent.config.model = m.trim().to_string();
            agent.model_override = None;
            println!("model = {}", agent.config.model);
            continue;
        }
        if let Some(goal) = line.strip_prefix("/plan ") {
            match run_plan(&mut agent, goal).await {
                Ok(p) => println!("{}", serde_json::to_string_pretty(&p)?),
                Err(e) => eprintln!("plan error: {e:#}"),
            }
            continue;
        }
        if let Some(goal) = line.strip_prefix("/mission ") {
            let client = agent.client.clone();
            let store2 = agent.store.clone();
            let tools = agent.tools.clone();
            let config = agent.config.clone();
            let cwd = agent.cwd.clone();
            let sid = store2.create_session(&cwd, &config.model)?;
            let mut boss = AgentLoop::new(client, store2, tools, config, cwd, sid.id);
            boss.permission = agent.permission;
            boss.ask_fn = agent.ask_fn.clone();
            boss.use_streaming = agent.use_streaming;
            match run_mission(
                boss,
                goal,
                MissionOptions {
                    auto_approve_graph: true,
                    ..Default::default()
                },
            )
            .await
            {
                Ok(o) => println!("{o}"),
                Err(e) => eprintln!("mission error: {e:#}"),
            }
            continue;
        }
        if line.starts_with('/') {
            eprintln!(
                "unknown command — try /plan /mission /yolo /cost /compact /model /clear /quit"
            );
            continue;
        }
        if let Err(e) = agent.run_turn(line).await {
            eprintln!("{} {e:#}", style("error:").red());
        }
        println!();
    }
    Ok(())
}

fn init_tracing(verbose: bool) {
    let filter = if verbose {
        EnvFilter::new("info,aegis=debug,aegis_core=debug,aegis_xai=debug,aegis_auth=debug")
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"))
    };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() <= n {
        s.to_string()
    } else {
        format!("{}…", &s[..n])
    }
}

// silence unused import warning if ToolRegistry only used via default
#[allow(dead_code)]
fn _tr(_: ToolRegistry) {}
