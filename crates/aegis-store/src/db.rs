use crate::paths::AegisPaths;
use anyhow::{Context, Result};
use chrono::Utc;
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use uuid::Uuid;

pub struct Store {
    conn: Mutex<Connection>,
    paths: AegisPaths,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    pub id: String,
    pub cwd: String,
    pub model: String,
    pub created_at: String,
    pub updated_at: String,
    pub title: Option<String>,
    pub previous_response_id: Option<String>,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub id: i64,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionRow {
    pub id: String,
    pub goal: String,
    pub status: String,
    pub graph_json: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRow {
    pub id: String,
    pub mission_id: String,
    pub title: String,
    pub status: String,
    pub depends_on: String,
    pub result: Option<String>,
    pub model_hint: Option<String>,
    pub needs_reasoning: bool,
}

impl Store {
    pub fn open(paths: &AegisPaths) -> Result<Self> {
        paths.ensure()?;
        let conn = Connection::open(&paths.db)
            .with_context(|| format!("open db {}", paths.db.display()))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let store = Self {
            conn: Mutex::new(conn),
            paths: paths.clone(),
        };
        store.migrate()?;
        Ok(store)
    }

    pub fn paths(&self) -> &AegisPaths {
        &self.paths
    }

    fn migrate(&self) -> Result<()> {
        self.conn.lock().execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                cwd TEXT NOT NULL,
                model TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                title TEXT,
                previous_response_id TEXT,
                total_input_tokens INTEGER NOT NULL DEFAULT 0,
                total_output_tokens INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS response_cache (
                cache_key TEXT PRIMARY KEY,
                response_json TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS missions (
                id TEXT PRIMARY KEY,
                session_id TEXT,
                goal TEXT NOT NULL,
                status TEXT NOT NULL,
                graph_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                mission_id TEXT NOT NULL REFERENCES missions(id) ON DELETE CASCADE,
                title TEXT NOT NULL,
                status TEXT NOT NULL,
                depends_on TEXT NOT NULL DEFAULT '[]',
                result TEXT,
                model_hint TEXT,
                needs_reasoning INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS artifacts (
                id TEXT PRIMARY KEY,
                mission_id TEXT NOT NULL,
                task_id TEXT,
                path TEXT NOT NULL,
                note TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                mission_id TEXT NOT NULL,
                task_id TEXT,
                body TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mission_id TEXT,
                kind TEXT NOT NULL,
                payload TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS todos (
                session_id TEXT NOT NULL,
                todos_json TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (session_id)
            );
            "#,
        )?;
        Ok(())
    }

    pub fn create_session(&self, cwd: &Path, model: &str) -> Result<SessionMeta> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let meta = SessionMeta {
            id: id.clone(),
            cwd: cwd.display().to_string(),
            model: model.to_string(),
            created_at: now.clone(),
            updated_at: now,
            title: None,
            previous_response_id: None,
            total_input_tokens: 0,
            total_output_tokens: 0,
        };
        self.conn.lock().execute(
            "INSERT INTO sessions (id, cwd, model, created_at, updated_at, title, previous_response_id, total_input_tokens, total_output_tokens)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                meta.id,
                meta.cwd,
                meta.model,
                meta.created_at,
                meta.updated_at,
                meta.title,
                meta.previous_response_id,
                meta.total_input_tokens as i64,
                meta.total_output_tokens as i64,
            ],
        )?;
        Ok(meta)
    }

    pub fn list_sessions(&self, limit: usize) -> Result<Vec<SessionMeta>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, cwd, model, created_at, updated_at, title, previous_response_id, total_input_tokens, total_output_tokens
             FROM sessions ORDER BY updated_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(SessionMeta {
                id: row.get(0)?,
                cwd: row.get(1)?,
                model: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                title: row.get(5)?,
                previous_response_id: row.get(6)?,
                total_input_tokens: row.get::<_, i64>(7)? as u64,
                total_output_tokens: row.get::<_, i64>(8)? as u64,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_session(&self, id: &str) -> Result<Option<SessionMeta>> {
        self.conn
            .lock()
            .query_row(
                "SELECT id, cwd, model, created_at, updated_at, title, previous_response_id, total_input_tokens, total_output_tokens
                 FROM sessions WHERE id = ?1",
                params![id],
                |row| {
                    Ok(SessionMeta {
                        id: row.get(0)?,
                        cwd: row.get(1)?,
                        model: row.get(2)?,
                        created_at: row.get(3)?,
                        updated_at: row.get(4)?,
                        title: row.get(5)?,
                        previous_response_id: row.get(6)?,
                        total_input_tokens: row.get::<_, i64>(7)? as u64,
                        total_output_tokens: row.get::<_, i64>(8)? as u64,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn set_previous_response_id(&self, session_id: &str, prev: Option<&str>) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "UPDATE sessions SET previous_response_id = ?1, updated_at = ?2 WHERE id = ?3",
            params![prev, now, session_id],
        )?;
        Ok(())
    }

    pub fn add_usage(&self, session_id: &str, input: u64, output: u64) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "UPDATE sessions SET total_input_tokens = total_input_tokens + ?1,
             total_output_tokens = total_output_tokens + ?2, updated_at = ?3 WHERE id = ?4",
            params![input as i64, output as i64, now, session_id],
        )?;
        Ok(())
    }

    pub fn append_message(&self, session_id: &str, role: &str, content: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO messages (session_id, role, content, created_at) VALUES (?1,?2,?3,?4)",
            params![session_id, role, content, now],
        )?;
        let now2 = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
            params![now2, session_id],
        )?;
        Ok(())
    }

    pub fn messages(&self, session_id: &str) -> Result<Vec<SessionMessage>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, role, content, created_at FROM messages WHERE session_id = ?1 ORDER BY id ASC",
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(SessionMessage {
                id: row.get(0)?,
                role: row.get(1)?,
                content: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn cache_get(&self, key: &str) -> Result<Option<String>> {
        self.conn
            .lock()
            .query_row(
                "SELECT response_json FROM response_cache WHERE cache_key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn cache_put(&self, key: &str, response_json: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "INSERT OR REPLACE INTO response_cache (cache_key, response_json, created_at) VALUES (?1,?2,?3)",
            params![key, response_json, now],
        )?;
        Ok(())
    }

    pub fn cache_key(parts: &[&str]) -> String {
        let mut hasher = Sha256::new();
        for p in parts {
            hasher.update(p.as_bytes());
            hasher.update([0]);
        }
        hex::encode(hasher.finalize())
    }

    pub fn set_todos(&self, session_id: &str, todos_json: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "INSERT OR REPLACE INTO todos (session_id, todos_json, updated_at) VALUES (?1,?2,?3)",
            params![session_id, todos_json, now],
        )?;
        Ok(())
    }

    pub fn get_todos(&self, session_id: &str) -> Result<Option<String>> {
        self.conn
            .lock()
            .query_row(
                "SELECT todos_json FROM todos WHERE session_id = ?1",
                params![session_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn create_mission(
        &self,
        session_id: Option<&str>,
        goal: &str,
        graph_json: &str,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "INSERT INTO missions (id, session_id, goal, status, graph_json, created_at, updated_at)
             VALUES (?1,?2,?3,'planning',?4,?5,?6)",
            params![id, session_id, goal, graph_json, now, now],
        )?;
        Ok(id)
    }

    pub fn update_mission_status(&self, id: &str, status: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "UPDATE missions SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, now, id],
        )?;
        Ok(())
    }

    pub fn update_mission_graph(&self, id: &str, graph_json: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "UPDATE missions SET graph_json = ?1, updated_at = ?2 WHERE id = ?3",
            params![graph_json, now, id],
        )?;
        Ok(())
    }

    pub fn upsert_task(&self, task: &TaskRow) -> Result<()> {
        self.conn.lock().execute(
            "INSERT INTO tasks (id, mission_id, title, status, depends_on, result, model_hint, needs_reasoning)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
             ON CONFLICT(id) DO UPDATE SET
               title=excluded.title, status=excluded.status, depends_on=excluded.depends_on,
               result=excluded.result, model_hint=excluded.model_hint, needs_reasoning=excluded.needs_reasoning",
            params![
                task.id,
                task.mission_id,
                task.title,
                task.status,
                task.depends_on,
                task.result,
                task.model_hint,
                task.needs_reasoning as i64,
            ],
        )?;
        Ok(())
    }

    pub fn set_task_status(&self, id: &str, status: &str, result: Option<&str>) -> Result<()> {
        self.conn.lock().execute(
            "UPDATE tasks SET status = ?1, result = COALESCE(?2, result) WHERE id = ?3",
            params![status, result, id],
        )?;
        Ok(())
    }

    pub fn list_tasks(&self, mission_id: &str) -> Result<Vec<TaskRow>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, mission_id, title, status, depends_on, result, model_hint, needs_reasoning
             FROM tasks WHERE mission_id = ?1",
        )?;
        let rows = stmt.query_map(params![mission_id], |row| {
            Ok(TaskRow {
                id: row.get(0)?,
                mission_id: row.get(1)?,
                title: row.get(2)?,
                status: row.get(3)?,
                depends_on: row.get(4)?,
                result: row.get(5)?,
                model_hint: row.get(6)?,
                needs_reasoning: row.get::<_, i64>(7)? != 0,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn add_note(&self, mission_id: &str, task_id: Option<&str>, body: &str) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "INSERT INTO notes (id, mission_id, task_id, body, created_at) VALUES (?1,?2,?3,?4,?5)",
            params![id, mission_id, task_id, body, now],
        )?;
        Ok(())
    }

    pub fn list_notes(&self, mission_id: &str) -> Result<Vec<(String, Option<String>, String)>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, task_id, body FROM notes WHERE mission_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map(params![mission_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn add_event(&self, mission_id: Option<&str>, kind: &str, payload: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "INSERT INTO events (mission_id, kind, payload, created_at) VALUES (?1,?2,?3,?4)",
            params![mission_id, kind, payload, now],
        )?;
        Ok(())
    }

    pub fn add_artifact(
        &self,
        mission_id: &str,
        task_id: Option<&str>,
        path: &str,
        note: Option<&str>,
    ) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "INSERT INTO artifacts (id, mission_id, task_id, path, note, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![id, mission_id, task_id, path, note, now],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::AegisPaths;
    use std::path::PathBuf;

    fn open_temp_store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let paths = AegisPaths::from_root(dir.path().to_path_buf());
        let store = Store::open(&paths).unwrap();
        (dir, store)
    }

    #[test]
    fn create_and_list_sessions() {
        let (_dir, store) = open_temp_store();
        let a = store
            .create_session(Path::new("/tmp/proj-a"), "grok-4.5")
            .unwrap();
        let b = store
            .create_session(Path::new("/tmp/proj-b"), "grok-code-fast-1")
            .unwrap();
        assert!(!a.id.is_empty());
        assert_eq!(a.model, "grok-4.5");
        assert_eq!(a.total_input_tokens, 0);

        let listed = store.list_sessions(10).unwrap();
        assert_eq!(listed.len(), 2);
        let ids: Vec<_> = listed.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&a.id.as_str()));
        assert!(ids.contains(&b.id.as_str()));

        let got = store.get_session(&a.id).unwrap().unwrap();
        assert_eq!(got.cwd, PathBuf::from("/tmp/proj-a").display().to_string());
        assert!(store.get_session("missing").unwrap().is_none());
    }

    #[test]
    fn list_sessions_respects_limit() {
        let (_dir, store) = open_temp_store();
        for i in 0..5 {
            store
                .create_session(Path::new(&format!("/tmp/p{i}")), "m")
                .unwrap();
        }
        assert_eq!(store.list_sessions(2).unwrap().len(), 2);
        assert_eq!(store.list_sessions(100).unwrap().len(), 5);
    }

    #[test]
    fn messages_append_and_order() {
        let (_dir, store) = open_temp_store();
        let s = store.create_session(Path::new("/w"), "m").unwrap();
        store.append_message(&s.id, "user", "hello").unwrap();
        store.append_message(&s.id, "assistant", "hi").unwrap();
        store.append_message(&s.id, "user", "more").unwrap();

        let msgs = store.messages(&s.id).unwrap();
        assert_eq!(msgs.len(), 3);
        assert_eq!(msgs[0].role, "user");
        assert_eq!(msgs[0].content, "hello");
        assert_eq!(msgs[1].role, "assistant");
        assert_eq!(msgs[2].content, "more");
        assert!(msgs[0].id < msgs[1].id);

        assert!(store.messages("no-such-session").unwrap().is_empty());
    }

    #[test]
    fn previous_response_id_and_usage() {
        let (_dir, store) = open_temp_store();
        let s = store.create_session(Path::new("/w"), "m").unwrap();
        store
            .set_previous_response_id(&s.id, Some("resp_abc"))
            .unwrap();
        let got = store.get_session(&s.id).unwrap().unwrap();
        assert_eq!(got.previous_response_id.as_deref(), Some("resp_abc"));

        store.add_usage(&s.id, 100, 50).unwrap();
        store.add_usage(&s.id, 20, 10).unwrap();
        let got = store.get_session(&s.id).unwrap().unwrap();
        assert_eq!(got.total_input_tokens, 120);
        assert_eq!(got.total_output_tokens, 60);

        store.set_previous_response_id(&s.id, None).unwrap();
        let got = store.get_session(&s.id).unwrap().unwrap();
        assert!(got.previous_response_id.is_none());
    }

    #[test]
    fn cache_put_get_and_overwrite() {
        let (_dir, store) = open_temp_store();
        assert!(store.cache_get("k1").unwrap().is_none());
        store.cache_put("k1", r#"{"id":"r1"}"#).unwrap();
        assert_eq!(
            store.cache_get("k1").unwrap().as_deref(),
            Some(r#"{"id":"r1"}"#)
        );
        store.cache_put("k1", r#"{"id":"r2"}"#).unwrap();
        assert_eq!(
            store.cache_get("k1").unwrap().as_deref(),
            Some(r#"{"id":"r2"}"#)
        );
    }

    #[test]
    fn cache_key_deterministic_and_order_sensitive() {
        let a = Store::cache_key(&["model", "prompt", "tools"]);
        let b = Store::cache_key(&["model", "prompt", "tools"]);
        let c = Store::cache_key(&["prompt", "model", "tools"]);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.len(), 64); // sha256 hex
    }

    #[test]
    fn todos_set_get_replace() {
        let (_dir, store) = open_temp_store();
        let s = store.create_session(Path::new("/w"), "m").unwrap();
        assert!(store.get_todos(&s.id).unwrap().is_none());
        store
            .set_todos(&s.id, r#"[{"id":1,"text":"do thing"}]"#)
            .unwrap();
        assert_eq!(
            store.get_todos(&s.id).unwrap().as_deref(),
            Some(r#"[{"id":1,"text":"do thing"}]"#)
        );
        store.set_todos(&s.id, r#"[]"#).unwrap();
        assert_eq!(store.get_todos(&s.id).unwrap().as_deref(), Some(r#"[]"#));
    }

    #[test]
    fn mission_tasks_notes_artifacts() {
        let (_dir, store) = open_temp_store();
        let s = store.create_session(Path::new("/w"), "m").unwrap();
        let mid = store
            .create_mission(Some(&s.id), "ship it", r#"{"tasks":[]}"#)
            .unwrap();
        store.update_mission_status(&mid, "running").unwrap();
        store
            .update_mission_graph(&mid, r#"{"tasks":[{"id":"t1"}]}"#)
            .unwrap();

        let task = TaskRow {
            id: "t1".into(),
            mission_id: mid.clone(),
            title: "Implement".into(),
            status: "pending".into(),
            depends_on: "[]".into(),
            result: None,
            model_hint: Some("grok-code-fast-1".into()),
            needs_reasoning: false,
        };
        store.upsert_task(&task).unwrap();
        store
            .set_task_status("t1", "done", Some("ok result"))
            .unwrap();
        let tasks = store.list_tasks(&mid).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].status, "done");
        assert_eq!(tasks[0].result.as_deref(), Some("ok result"));

        store.add_note(&mid, Some("t1"), "blackboard note").unwrap();
        store.add_note(&mid, None, "global note").unwrap();
        let notes = store.list_notes(&mid).unwrap();
        assert_eq!(notes.len(), 2);

        store
            .add_artifact(&mid, Some("t1"), "/tmp/out.txt", Some("built"))
            .unwrap();
        store
            .add_event(Some(&mid), "task_done", r#"{"id":"t1"}"#)
            .unwrap();
    }

    #[test]
    fn paths_from_root_and_ensure() {
        let dir = tempfile::tempdir().unwrap();
        let paths = AegisPaths::from_root(dir.path().to_path_buf());
        assert_eq!(paths.db, dir.path().join("aegis.db"));
        assert_eq!(paths.artifacts, dir.path().join("artifacts"));
        paths.ensure().unwrap();
        assert!(paths.root.is_dir());
        assert!(paths.artifacts.is_dir());
    }
}
