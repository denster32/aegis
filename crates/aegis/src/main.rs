use aegis_auth::{
    auth_paths, clear_auth_file, device_login, import_grok_to_aegis, AuthProvider, TokenSource,
};
use aegis_core::{
    assess_v2, automations, checkpoint_create, checkpoint_list, checkpoint_restore, factory_status,
    format_factory, format_readiness_v2, generate_wiki, install_dream_cron, install_qa,
    install_review_workflow, install_wiki_workflow, missions_new, missions_run, missions_status,
    readiness_report, review_diff, review_pr, run_dream, run_mission, run_plan, run_qa, ui,
    AegisConfig, AgentLoop, DreamOptions, Effort, MissionOptions,
};
use aegis_evolution::{
    list_run_ids, load_run, save_run, score_run, EvolutionEngine, FitnessSignals, MutationRequest,
};
use aegis_hardware::{format_probe, policy_from_snapshot, probe_host};
use aegis_memory::{NeuralSummary, ProjectMemory};
use aegis_spore::{pack_spore, unpack_spore, vaccinate};
use aegis_store::{AegisPaths, Store};
use aegis_tools::{default_registry, PermissionMode};
use aegis_xai::{ResponsesClient, TokenSource as XaiTokenSource};
use anyhow::{Context, Result};
use async_trait::async_trait;
use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{Parser, Subcommand};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

/// SpaceX / xAI monochrome clap palette.
fn clap_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::White.on_default().effects(Effects::BOLD))
        .usage(AnsiColor::White.on_default().effects(Effects::BOLD))
        .literal(AnsiColor::White.on_default())
        .placeholder(AnsiColor::BrightBlack.on_default())
        .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
        .valid(AnsiColor::White.on_default())
        .invalid(AnsiColor::Red.on_default())
}

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
    about = "Sovereign Grok coding agent",
    long_about = "AEGIS — sovereign Grok-native coding agent in Rust.\n\
                  Black · white · precise. Tools · Missions · learning.",
    styles = clap_styles(),
    after_help = "Docs · https://github.com/denster32/aegis"
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

    /// Sandbox: deny shell; workspace-only FS (no outside-cwd approval). Overrides --yolo / auto-yolo.
    #[arg(long, global = true)]
    sandbox: bool,

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
    // ── Agent ──────────────────────────────────────────────
    /// Structured plan only
    #[command(next_help_heading = "Agent")]
    Plan { goal: String },
    /// Multi-agent DAG swarm mission
    Mission {
        goal: String,
        #[arg(long, default_value_t = 4)]
        workers: usize,
        #[arg(long)]
        approve: bool,
    },
    /// Factory Missions (plan → control → execute)
    Missions {
        #[command(subcommand)]
        action: MissionsCmd,
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

    // ── Platform ───────────────────────────────────────────
    /// Agent readiness L1–L5
    #[command(next_help_heading = "Platform")]
    Readiness {
        #[arg(long)]
        json: bool,
        #[arg(long, default_value_t = true)]
        v2: bool,
    },
    /// Software Factory SDLC coverage map
    Factory,
    /// Nightly dream — deep self-improve
    Dream {
        #[arg(long)]
        apply: bool,
        #[arg(long, default_value = "high")]
        budget: String,
        #[command(subcommand)]
        action: Option<DreamCmd>,
    },
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
    /// Run Automated QA
    Qa {
        #[arg(long)]
        base: Option<String>,
    },
    /// File-based automations
    Automation {
        #[command(subcommand)]
        action: AutomationCmd,
    },

    // ── Install ────────────────────────────────────────────
    /// Install Automated QA skills
    #[command(next_help_heading = "Install")]
    InstallQa,
    /// Install GH code-review workflow
    InstallCodeReview,
    /// Install GH wiki-refresh workflow
    InstallWikiRefresh,
    /// Live smoke harness (`scripts/live_smoke.sh`) — no agent loop
    Smoke,
    /// Live stress harness (`scripts/stress_test.sh`) — no agent loop
    Stress {
        /// Run S15–S20 long band (default true; set STRESS_LONG=0 to skip)
        #[arg(long, default_value_t = true)]
        long: bool,
    },

    // ── Nexus ──────────────────────────────────────────────
    /// Nexus organism status (cells · membrane · summary)
    #[command(next_help_heading = "Nexus")]
    Nexus {
        #[command(subcommand)]
        action: Option<NexusCmd>,
    },
    /// Evolution: Grok mutation genes + local fitness
    Evolve {
        #[command(subcommand)]
        action: EvolveCmd,
    },
    /// Viral spore pack / unpack / vaccinate
    Spore {
        #[command(subcommand)]
        action: SporeCmd,
    },
    /// Neural summary compression (local or Grok)
    Compress {
        /// Use local distillation only (no API)
        #[arg(long)]
        local: bool,
    },
    /// Hardware probe and throttle policy
    Hardware {
        #[command(subcommand)]
        action: HardwareCmd,
    },

    // ── Auth ───────────────────────────────────────────────
    /// Authenticate with Grok OAuth
    #[command(next_help_heading = "Auth")]
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

#[derive(Subcommand, Debug)]
enum NexusCmd {
    /// Organism overview (default)
    Status,
}

#[derive(Subcommand, Debug)]
enum EvolveCmd {
    /// Propose genes for a goal (Grok structured)
    Propose {
        goal: String,
        #[arg(long, default_value_t = 3)]
        max: usize,
    },
    /// Score a run's genes with local fitness signals
    Run { id: String },
    /// List evolution runs
    Status,
}

#[derive(Subcommand, Debug)]
enum SporeCmd {
    /// Pack redacted learning into a spore directory
    Pack {
        #[arg(long, default_value = ".aegis/nexus/spore-out")]
        out: PathBuf,
    },
    /// Unpack spore into current project
    Unpack { path: PathBuf },
    /// Unpack + force sandbox=true
    Vaccinate { path: PathBuf },
}

#[derive(Subcommand, Debug)]
enum HardwareCmd {
    /// Probe host (cpu/mem/load)
    Probe,
    /// Derive throttle policy from probe
    Policy,
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
                    println!("{}", ui::event("memory", "cleared lessons + failures"));
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
        Some(Commands::Smoke) => {
            return run_harness_script(&cwd_early, "live_smoke.sh", &[]);
        }
        Some(Commands::Stress { long }) => {
            let long_env = if *long { "1" } else { "0" };
            return run_harness_script(&cwd_early, "stress_test.sh", &[("STRESS_LONG", long_env)]);
        }
        Some(Commands::Nexus { action }) => {
            let _ = action;
            print!("{}", nexus_status(&cwd_early, cli.sandbox)?);
            return Ok(());
        }
        Some(Commands::Hardware { action }) => {
            let snap = probe_host();
            match action {
                HardwareCmd::Probe => {
                    println!("{}", ui::header("hardware"));
                    print!("{}", format_probe(&snap));
                }
                HardwareCmd::Policy => {
                    let pol = policy_from_snapshot(&snap);
                    println!("{}", ui::header("hardware policy"));
                    println!("{}", ui::kv("parallel", pol.max_tool_parallel.to_string()));
                    println!("{}", ui::kv("effort", &pol.preferred_effort));
                    println!("{}", ui::kv("max_steps", pol.max_agent_steps.to_string()));
                    println!("{}", ui::kv("notes", &pol.notes));
                }
            }
            return Ok(());
        }
        Some(Commands::Spore { action }) => {
            match action {
                SporeCmd::Pack { out } => {
                    let out = if out.is_absolute() {
                        out.clone()
                    } else {
                        cwd_early.join(out)
                    };
                    let m = pack_spore(&cwd_early, &out)?;
                    println!("{}", ui::header("spore pack"));
                    println!("{}", ui::kv("out", out.display().to_string()));
                    println!("{}", ui::kv("sandbox", m.sandbox_default.to_string()));
                    println!("{}", ui::kv("includes", m.includes.join(", ")));
                }
                SporeCmd::Unpack { path } => {
                    let p = if path.is_absolute() {
                        path.clone()
                    } else {
                        cwd_early.join(path)
                    };
                    unpack_spore(&p, &cwd_early)?;
                    println!("{}", ui::event("spore", "unpacked"));
                }
                SporeCmd::Vaccinate { path } => {
                    let p = if path.is_absolute() {
                        path.clone()
                    } else {
                        cwd_early.join(path)
                    };
                    vaccinate(&p, &cwd_early)?;
                    println!("{}", ui::event("spore", "vaccinated (sandbox=true)"));
                }
            }
            return Ok(());
        }
        Some(Commands::Compress { local: true }) => {
            let mem = ProjectMemory::open(&cwd_early)?;
            let sum = NeuralSummary::from_project_local(&mem)?;
            sum.save(&cwd_early)?;
            println!("{}", ui::header("compress"));
            println!("{}", ui::kv("mode", "local"));
            println!(
                "{}",
                ui::kv(
                    "path",
                    NeuralSummary::path(&cwd_early).display().to_string()
                )
            );
            println!(
                "{}",
                ui::kv("lessons", sum.durable_lessons.len().to_string())
            );
            return Ok(());
        }
        Some(Commands::Evolve {
            action: EvolveCmd::Status,
        }) => {
            println!("{}", ui::header("evolve"));
            let ids = list_run_ids(&cwd_early).unwrap_or_default();
            if ids.is_empty() {
                println!("{}", ui::empty("none"));
            } else {
                for id in ids {
                    if let Ok(run) = load_run(&cwd_early, &id) {
                        println!(
                            "  {}  genes={}  best={}",
                            ui::primary(&id[..8.min(id.len())]),
                            run.genes.len(),
                            run.best_gene_id.as_deref().unwrap_or("—")
                        );
                    }
                }
            }
            return Ok(());
        }
        Some(Commands::Evolve {
            action: EvolveCmd::Run { id },
        }) => {
            let run = load_run(&cwd_early, id)?;
            let report = assess_v2(&cwd_early);
            let signals = FitnessSignals {
                readiness_pct: report.score_pct as f32,
                cargo_ok: Some(
                    std::process::Command::new("cargo")
                        .args(["test", "--quiet"])
                        .current_dir(&cwd_early)
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false),
                ),
                lesson_hits: ProjectMemory::open(&cwd_early)
                    .ok()
                    .and_then(|m| m.load_lessons().ok())
                    .map(|l| l.len() as u32)
                    .unwrap_or(0),
            };
            let scored = score_run(&run.goal, run.genes.clone(), &signals);
            save_run(&cwd_early, &scored)?;
            println!("{}", ui::header("evolve run"));
            println!("{}", ui::kv("id", &scored.id));
            println!(
                "{}",
                ui::kv("best", scored.best_gene_id.as_deref().unwrap_or("—"))
            );
            for s in &scored.scores {
                println!(
                    "  {}  {:.2}  {}",
                    ui::primary(&s.gene_id[..8.min(s.gene_id.len())]),
                    s.score,
                    ui::dim(&s.notes)
                );
            }
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
            println!(
                "{}",
                ui::event("automations", "defaults ensured under .aegis/automations/")
            );
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
            println!("{}", ui::header("sessions"));
            for s in store.list_sessions(*limit)? {
                println!(
                    "  {}  {}  {}",
                    ui::primary(&s.id[..8.min(s.id.len())]),
                    ui::dim(&s.model),
                    ui::dim(format!(
                        "in={} out={}",
                        s.total_input_tokens, s.total_output_tokens
                    ))
                );
                println!("      {}  {}", ui::dim(&s.updated_at), ui::dim(&s.cwd));
            }
            return Ok(());
        }
        Some(Commands::Session {
            action: SessionCmd::Show { id },
        }) => {
            let s = resolve_session(&store, id)?;
            println!("{}", ui::header("session"));
            println!("{}", ui::kv("id", &s.id));
            println!("{}", ui::kv("model", &s.model));
            println!("{}", ui::kv("cwd", &s.cwd));
            println!(
                "{}",
                ui::kv(
                    "tokens",
                    format!("in={} out={}", s.total_input_tokens, s.total_output_tokens)
                )
            );
            println!(
                "{}",
                ui::kv("prev", s.previous_response_id.as_deref().unwrap_or("—"))
            );
            println!("{}", ui::rule());
            for m in store.messages(&s.id)? {
                println!(
                    "  {}  {}  {}",
                    ui::dim(&m.role),
                    ui::dim(&m.created_at),
                    ui::primary(truncate(&m.content, 240))
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
                println!("{}", ui::header("login"));
                println!("{}", ui::kv("action", "import grok → aegis"));
                println!("{}", ui::kv("email", e.email.as_deref().unwrap_or("—")));
                println!(
                    "{}",
                    ui::kv("expires", e.expires_at.as_deref().unwrap_or("—"))
                );
                return Ok(());
            }
            let ap = auth_paths();
            let entry = device_login(&ap, !*write_grok).await?;
            println!("{}", ui::header("login"));
            println!(
                "{}",
                ui::kv("source", if *write_grok { "grok" } else { "aegis" })
            );
            println!("{}", ui::kv("email", entry.email.as_deref().unwrap_or("—")));
            println!(
                "{}",
                ui::kv("expires", entry.expires_at.as_deref().unwrap_or("—"))
            );
            return Ok(());
        }
        Some(Commands::Logout) => {
            let ap = auth_paths();
            clear_auth_file(&ap.aegis)?;
            println!("{}", ui::header("logout"));
            println!("{}", ui::kv("cleared", ap.aegis.display().to_string()));
            println!(
                "  {}",
                ui::dim("~/.grok/auth.json left intact — grok logout to clear")
            );
            return Ok(());
        }
        Some(Commands::Auth {
            action: AuthCmd::Status,
        }) => {
            match AuthProvider::resolve() {
                Ok(p) => {
                    let st = p.status();
                    print!(
                        "{}",
                        ui::auth_status(
                            st.source.as_str(),
                            st.email.as_deref().unwrap_or("—"),
                            st.expires_at.as_deref().unwrap_or("—"),
                            st.team_id.as_deref().unwrap_or("—"),
                            st.auth_mode.as_deref().unwrap_or("—"),
                            st.path.as_deref().unwrap_or("—"),
                            st.needs_refresh,
                        )
                    );
                }
                Err(e) => {
                    print!("{}", ui::auth_unsigned(&format!("{e:#}")));
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
            println!("{}", ui::header("auth"));
            println!("{}", ui::kv("action", "refresh"));
            println!(
                "{}",
                ui::kv("expires", st.expires_at.as_deref().unwrap_or("—"))
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
    if cli.sandbox {
        config.sandbox = true;
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
            eprintln!("{}", ui::warn_line(format!("MCP: {e:#}")));
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
    // Sandbox always wins over yolo / auto-yolo.
    let auto_yolo = config.yolo
        || cli.prompt.is_some()
        || matches!(
            cli.command,
            Some(Commands::Mission { .. })
                | Some(Commands::Plan { .. })
                | Some(Commands::Missions { .. })
        );
    agent.permission = if config.sandbox {
        PermissionMode::Deny
    } else if auto_yolo {
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
            println!("\n{}", ui::footer_done());
            println!("{out}");
        }
        Some(Commands::Plan { goal }) => {
            println!("{}", ui::header("plan"));
            let plan = run_plan(&mut agent, &goal).await?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
            let _ = agent.reflect_and_save().await;
        }
        Some(Commands::Missions {
            action: MissionsCmd::New { goal, oneshot },
        }) => {
            println!("{}", ui::header("missions"));
            let plan = missions_new(&mut agent, &goal, oneshot).await?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
            println!("{}", ui::rule());
            println!("{}", ui::kv("saved", &plan.id));
            let short = &plan.id[..8.min(plan.id.len())];
            println!("{}", ui::kv("next", format!("aegis missions run {short}")));
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
            println!("{}", ui::header("dream"));
            println!("{}", ui::kv("phase", "run"));
            let journal = run_dream(&agent.client, &cwd, opts).await?;
            println!("{}", ui::kv("id", &journal.id));
            println!("{}", ui::kv("applied", format!("{:?}", journal.applied)));
            println!(
                "{}",
                ui::kv("proposals", journal.proposals.len().to_string())
            );
            println!("{}", ui::rule());
            for p in &journal.proposals {
                println!(
                    "  {}  {}  {}",
                    ui::mark_idle(),
                    ui::dim(&p.kind),
                    ui::primary(&p.title)
                );
            }
            println!("\n  {}", ui::dim("journal · .aegis/dreams/"));
        }
        Some(Commands::Wiki { action }) => {
            let model = config.model.clone();
            match action {
                WikiCmd::Generate | WikiCmd::Refresh => {
                    println!("{}", ui::header("wiki"));
                    let n = generate_wiki(&cwd, &agent.client, &model).await?;
                    println!("{}", ui::kv("pages", n.to_string()));
                    println!("{}", ui::kv("path", "docs/wiki/"));
                }
            }
        }
        Some(Commands::Review { pr, diff, depth }) => {
            let model = config.model.clone();
            println!("{}", ui::header("review"));
            let report = if let Some(n) = pr {
                review_pr(&agent.client, &model, &cwd, n, &depth).await?
            } else if diff {
                review_diff(&agent.client, &model, &cwd, &depth).await?
            } else {
                anyhow::bail!("pass --pr N or --diff");
            };
            println!("{}", ui::kv("summary", &report.summary));
            println!(
                "{}",
                ui::kv("approve", if report.approve { "yes" } else { "no" })
            );
            println!("{}", ui::rule());
            for f in &report.findings {
                println!(
                    "  {}  {}  {}  {}",
                    ui::mark_idle(),
                    ui::dim(&f.severity),
                    ui::primary(&f.title),
                    ui::dim(&f.detail)
                );
            }
        }
        Some(Commands::Checkpoint { action }) => match action {
            CheckpointCmd::Create { label } => {
                let cp = checkpoint_create(&cwd, &label)?;
                println!("{}", ui::header("checkpoint"));
                println!("{}", ui::kv("id", &cp.id));
                println!("{}", ui::kv("stash", format!("{:?}", cp.stash_ref)));
            }
            CheckpointCmd::List => {
                println!("{}", ui::header("checkpoints"));
                for c in checkpoint_list(&cwd)? {
                    println!(
                        "  {}  {}  {}",
                        ui::primary(&c.id),
                        ui::dim(&c.label),
                        ui::dim(&c.created_at)
                    );
                }
            }
            CheckpointCmd::Restore { id } => {
                println!("{}", ui::header("checkpoint"));
                println!("{}", checkpoint_restore(&cwd, &id)?);
            }
        },
        Some(Commands::Vision { path, question }) => {
            let p = std::path::PathBuf::from(&path);
            let p = if p.is_absolute() { p } else { cwd.join(p) };
            println!("{}", ui::header("vision"));
            let out = aegis_tools::describe_image_file(&p, &question).await?;
            println!("{out}");
        }
        Some(Commands::InstallQa) => {
            println!("{}", ui::header("install"));
            println!("{}", install_qa(&cwd)?);
        }
        Some(Commands::Qa { base }) => {
            println!("{}", ui::header("qa"));
            println!("{}", run_qa(&cwd, base.as_deref())?);
        }
        Some(Commands::InstallCodeReview) => {
            let p = install_review_workflow(&cwd)?;
            println!("{}", ui::header("install"));
            println!("{}", ui::kv("workflow", p.display().to_string()));
        }
        Some(Commands::InstallWikiRefresh) => {
            let p = install_wiki_workflow(&cwd)?;
            println!("{}", ui::header("install"));
            println!("{}", ui::kv("workflow", p.display().to_string()));
        }
        Some(Commands::Automation {
            action: AutomationCmd::Run { name },
        }) => {
            println!("{}", ui::header("automation"));
            println!("{}", ui::kv("run", &name));
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
            println!("{}", ui::header("install"));
            println!("{}", ui::event("ok", "automations · workflows · skills"));
        }
        Some(Commands::Compress { local: false }) => {
            // Prefer Grok distillation; fall back to local on API failure.
            println!("{}", ui::header("compress"));
            let mem = ProjectMemory::open(&cwd)?;
            let pack = format!(
                "MEMORY:\n{}\n\nLESSONS:\n{:#?}",
                mem.read_memory_md().unwrap_or_default(),
                mem.load_lessons().unwrap_or_default()
            );
            let mut agent_c = agent;
            let prompt = format!(
                "Distill this project immune memory into JSON with keys: \
                 stack (string), conventions (string[]), active_risks (string[]), \
                 durable_lessons (string[]), style_priors (string[]), open_threads (string[]), narrative (string).\n\n{pack}"
            );
            match agent_c.run_turn(&prompt).await {
                Ok(text) => {
                    let mut sum = NeuralSummary::from_project_local(&mem)?;
                    if let Some(json) = extract_json_obj(&text) {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json) {
                            if let Some(s) = v.get("stack").and_then(|x| x.as_str()) {
                                sum.stack = s.into();
                            }
                            if let Some(s) = v.get("narrative").and_then(|x| x.as_str()) {
                                sum.narrative = s.into();
                            }
                            if let Some(a) = v.get("durable_lessons").and_then(|x| x.as_array()) {
                                sum.durable_lessons = a
                                    .iter()
                                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                                    .collect();
                            }
                            if let Some(a) = v.get("active_risks").and_then(|x| x.as_array()) {
                                sum.active_risks = a
                                    .iter()
                                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                                    .collect();
                            }
                            if let Some(a) = v.get("conventions").and_then(|x| x.as_array()) {
                                sum.conventions = a
                                    .iter()
                                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                                    .collect();
                            }
                        }
                    }
                    sum.save(&cwd)?;
                    println!("{}", ui::kv("mode", "grok+local-merge"));
                    println!(
                        "{}",
                        ui::kv("path", NeuralSummary::path(&cwd).display().to_string())
                    );
                }
                Err(e) => {
                    let sum = NeuralSummary::from_project_local(&mem)?;
                    sum.save(&cwd)?;
                    println!("{}", ui::kv("mode", "local-fallback"));
                    println!("{}", ui::warn_line(format!("grok failed: {e:#}")));
                }
            }
        }
        Some(Commands::Evolve {
            action: EvolveCmd::Propose { goal, max },
        }) => {
            println!("{}", ui::header("evolve propose"));
            let mem = ProjectMemory::open(&cwd).ok();
            let ctx = mem
                .as_ref()
                .and_then(|m| m.read_memory_md().ok())
                .unwrap_or_default();
            let engine = EvolutionEngine::new(agent.client.clone(), agent.config.model.clone());
            let req = MutationRequest {
                goal: goal.clone(),
                max_genes: max,
                context: truncate(&ctx, 4000),
            };
            let genes = engine.propose(&req).await?;
            let signals = FitnessSignals {
                readiness_pct: assess_v2(&cwd).score_pct as f32,
                cargo_ok: None,
                lesson_hits: mem
                    .and_then(|m| m.load_lessons().ok())
                    .map(|l| l.len() as u32)
                    .unwrap_or(0),
            };
            let run = score_run(goal, genes, &signals);
            save_run(&cwd, &run)?;
            println!("{}", ui::kv("run", &run.id));
            println!("{}", ui::kv("genes", run.genes.len().to_string()));
            for g in &run.genes {
                println!(
                    "  {}  {}  {}",
                    ui::primary(&g.id[..8.min(g.id.len())]),
                    ui::dim(format!("{:?}", g.kind)),
                    ui::primary(&g.title)
                );
            }
            println!(
                "{}",
                ui::kv(
                    "next",
                    format!("aegis evolve run {}", &run.id[..8.min(run.id.len())])
                )
            );
        }
        Some(Commands::Missions { .. })
        | Some(Commands::Memory { .. })
        | Some(Commands::Readiness { .. })
        | Some(Commands::Factory)
        | Some(Commands::Smoke)
        | Some(Commands::Stress { .. })
        | Some(Commands::Automation { .. })
        | Some(Commands::Session { .. })
        | Some(Commands::Login { .. })
        | Some(Commands::Logout)
        | Some(Commands::Auth { .. })
        | Some(Commands::Nexus { .. })
        | Some(Commands::Hardware { .. })
        | Some(Commands::Spore { .. })
        | Some(Commands::Compress { local: true })
        | Some(Commands::Evolve {
            action: EvolveCmd::Status | EvolveCmd::Run { .. },
        }) => unreachable!(),
        None => {
            if let Some(p) = cli.prompt {
                eprint!(
                    "{}",
                    ui::run_header(
                        &agent.config.model,
                        agent.config.reasoning_effort.as_str(),
                        &agent.session_id[..8.min(agent.session_id.len())],
                    )
                );
                let _ = agent.run_turn(&p).await?;
                let _ = agent.reflect_and_save().await;
                eprintln!("{}", ui::footer_done());
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
    print!(
        "{}",
        ui::repl_banner(
            env!("CARGO_PKG_VERSION"),
            &agent.session_id[..8.min(agent.session_id.len())],
            &agent.config.model,
            agent.config.reasoning_effort.as_str(),
            &agent.cwd.display().to_string(),
            agent.config.yolo,
            agent.config.sandbox,
        )
    );

    let stdin = io::stdin();
    loop {
        eprint!("{} ", ui::prompt_glyph());
        let _ = io::stderr().flush();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if matches!(line, "/quit" | "/exit" | "quit" | "exit" | ":q") {
            let _ = agent.reflect_and_save().await;
            println!("{}", ui::footer_done());
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
                    Ok(p) => {
                        println!("{}", ui::kv("saved", &p.id));
                        println!(
                            "{}",
                            ui::kv(
                                "next",
                                format!("aegis missions run {}", &p.id[..8.min(p.id.len())])
                            )
                        );
                    }
                    Err(e) => eprintln!("{}", ui::error_line(format!("{e:#}"))),
                }
            } else {
                eprintln!("{}", ui::note("/missions  list | status <id> | new <goal>"));
            }
            continue;
        }
        if line == "/yolo" {
            if agent.config.sandbox || matches!(agent.permission, PermissionMode::Deny) {
                println!("{}", ui::event("mode", "sandbox (yolo blocked)"));
            } else {
                agent.permission = PermissionMode::Yolo;
                agent.config.yolo = true;
                println!("{}", ui::event("mode", "yolo"));
            }
            continue;
        }
        if line == "/cost" {
            if let Ok(Some(s)) = store.get_session(&agent.session_id) {
                println!("{}", ui::header("cost"));
                println!("{}", ui::kv("session", &s.id[..8.min(s.id.len())]));
                println!("{}", ui::kv("in", s.total_input_tokens.to_string()));
                println!("{}", ui::kv("out", s.total_output_tokens.to_string()));
                println!(
                    "{}",
                    ui::kv(
                        "total",
                        (s.total_input_tokens + s.total_output_tokens).to_string()
                    )
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
                Ok(_) => println!("{}", ui::event("compact", "ok")),
                Err(e) => eprintln!("{}", ui::error_line(format!("{e:#}"))),
            }
            continue;
        }
        if line == "/clear" {
            agent.previous_response_id = None;
            let _ = store.set_previous_response_id(&agent.session_id, None);
            println!("{}", ui::event("clear", "server chain reset"));
            continue;
        }
        if let Some(m) = line.strip_prefix("/model ") {
            agent.config.model = m.trim().to_string();
            agent.model_override = None;
            println!("{}", ui::kv("model", &agent.config.model));
            continue;
        }
        if let Some(goal) = line.strip_prefix("/plan ") {
            match run_plan(&mut agent, goal).await {
                Ok(p) => println!("{}", serde_json::to_string_pretty(&p)?),
                Err(e) => eprintln!("{}", ui::error_line(format!("{e:#}"))),
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
                Err(e) => eprintln!("{}", ui::error_line(format!("{e:#}"))),
            }
            continue;
        }
        if line.starts_with('/') {
            eprintln!(
                "{}",
                ui::note(
                    "/plan /mission /missions /memory /yolo /cost /compact /model /clear /quit"
                )
            );
            continue;
        }
        if let Err(e) = agent.run_turn(line).await {
            eprintln!("{}", ui::error_line(format!("{e:#}")));
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
    let count = s.chars().count();
    if count <= n {
        s.to_string()
    } else {
        let mut t: String = s.chars().take(n.saturating_sub(1)).collect();
        t.push('…');
        t
    }
}

fn extract_json_obj(text: &str) -> Option<String> {
    let t = text.trim();
    if t.starts_with('{') {
        return Some(t.to_string());
    }
    let start = t.find('{')?;
    let end = t.rfind('}')?;
    if end > start {
        Some(t[start..=end].to_string())
    } else {
        None
    }
}

fn nexus_status(cwd: &std::path::Path, sandbox_flag: bool) -> Result<String> {
    use std::fmt::Write;
    let mut s = String::new();
    s.push_str(&ui::header("nexus"));
    let _ = writeln!(s, "{}", ui::kv("organism", "aegis-nexus"));
    let _ = writeln!(
        s,
        "{}",
        ui::kv(
            "membrane",
            if sandbox_flag {
                "sandbox"
            } else {
                "prompt/yolo"
            }
        )
    );
    let _ = writeln!(s, "{}", ui::kv("cwd", cwd.display().to_string()));
    if let Ok(mem) = ProjectMemory::open(cwd) {
        let lessons = mem.load_lessons().map(|l| l.len()).unwrap_or(0);
        let fails = mem.load_failures().map(|l| l.len()).unwrap_or(0);
        let _ = writeln!(s, "{}", ui::kv("lessons", lessons.to_string()));
        let _ = writeln!(s, "{}", ui::kv("failures", fails.to_string()));
    }
    match NeuralSummary::load(cwd)? {
        Some(n) => {
            let _ = writeln!(s, "{}", ui::kv("neural", &n.created_at));
            let _ = writeln!(
                s,
                "{}",
                ui::kv("lessons_n", n.durable_lessons.len().to_string())
            );
        }
        None => {
            let _ = writeln!(s, "{}", ui::kv("neural", "none — aegis compress"));
        }
    }
    let evo = list_run_ids(cwd).unwrap_or_default().len();
    let _ = writeln!(s, "{}", ui::kv("evolution", format!("{evo} runs")));
    let snap = probe_host();
    let _ = writeln!(
        s,
        "{}",
        ui::kv(
            "host",
            format!(
                "{} cpus={} load={:?}",
                snap.hostname, snap.cpus, snap.load_1
            )
        )
    );
    let reg = default_registry();
    let map = reg.capability_map();
    let _ = writeln!(
        s,
        "{}",
        ui::kv(
            "tools",
            map.get("tool_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .to_string()
        )
    );
    // snapshot capability map for inspection
    let cap_path = cwd.join(".aegis/nexus/capability-map.json");
    if let Some(p) = cap_path.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    let _ = std::fs::write(&cap_path, serde_json::to_string_pretty(&map)?);
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "  {}",
        ui::dim("evolve · spore · compress · hardware · --sandbox")
    );
    Ok(s)
}

/// Run repo harness scripts without entering the agent loop (no auth required).
fn run_harness_script(
    cwd: &std::path::Path,
    script_name: &str,
    extra_env: &[(&str, &str)],
) -> Result<()> {
    let script = find_harness_script(cwd, script_name).with_context(|| {
        format!(
            "could not find scripts/{script_name} under {} (run from the aegis checkout)",
            cwd.display()
        )
    })?;
    println!("{}", ui::header("harness"));
    println!("{}", ui::kv("script", script.display().to_string()));
    for (k, v) in extra_env {
        println!("{}", ui::kv(k, *v));
    }
    let mut cmd = std::process::Command::new("bash");
    cmd.arg(&script).current_dir(cwd);
    // Prefer the currently running binary for nested aegis calls.
    if let Ok(exe) = std::env::current_exe() {
        cmd.env("AEGIS", exe);
    }
    for (k, v) in extra_env {
        cmd.env(k, v);
    }
    let status = cmd
        .status()
        .with_context(|| format!("spawn {}", script.display()))?;
    if !status.success() {
        anyhow::bail!(
            "{script_name} failed with {}",
            status
                .code()
                .map(|c| format!("exit {c}"))
                .unwrap_or_else(|| "signal".into())
        );
    }
    Ok(())
}

fn find_harness_script(cwd: &std::path::Path, name: &str) -> Option<std::path::PathBuf> {
    let mut candidates = vec![cwd.join("scripts").join(name), cwd.join(name)];
    // Walk up a few parents (run from crates/* etc.)
    let mut cur = cwd.to_path_buf();
    for _ in 0..4 {
        if let Some(p) = cur.parent() {
            cur = p.to_path_buf();
            candidates.push(cur.join("scripts").join(name));
        }
    }
    candidates.into_iter().find(|p| p.is_file())
}
