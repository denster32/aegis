# QA Orchestrator

1. Read config.yaml
2. Map git diff to apps
3. Run qa-cli (or app-specific) flows
4. Write report under .aegis/qa/reports/
5. On failures, record to project FAILURES.jsonl via memory_write
