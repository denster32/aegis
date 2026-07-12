#!/usr/bin/env bash
# Publish docs/wiki → GitHub Wiki (https://github.com/OWNER/REPO/wiki)
#
# Prerequisites:
#   1. Repo Settings → Features → Wikis enabled
#   2. First page created once (empty "Home" is fine) OR use a classic PAT with `repo` scope
#   3. GH_TOKEN / gh auth with wiki write access
#
# Usage:
#   ./scripts/publish_wiki.sh
#   GH_TOKEN=ghp_xxx ./scripts/publish_wiki.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/docs/wiki"
REPO="${GITHUB_REPOSITORY:-denster32/aegis}"
TOKEN="${GH_TOKEN:-${GITHUB_TOKEN:-$(gh auth token 2>/dev/null || true)}}"
if [[ -z "${TOKEN}" ]]; then
  echo "error: no GH_TOKEN / gh auth token" >&2
  exit 1
fi

BLOB="https://github.com/${REPO}/blob/main"
TMP="$(mktemp -d /tmp/aegis-wiki-XXXXXX)"
cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT

prepare() {
  local out="$1"
  mkdir -p "$out"
  cp "$SRC/Home.md" "$out/Home.md"
  cp "$SRC/Architecture.md" "$out/Architecture.md"
  cp "$SRC/Modules.md" "$out/Modules.md"
  cp "$SRC/Commands.md" "$out/Commands.md"
  cp "$SRC/Conventions.md" "$out/Conventions.md"
  [[ -f "$SRC/Nexus.md" ]] && cp "$SRC/Nexus.md" "$out/Nexus.md"
  [[ -f "$ROOT/docs/nexus.md" && ! -f "$out/Nexus.md" ]] && cp "$ROOT/docs/nexus.md" "$out/Nexus.md"

  # GitHub Wiki uses page names without .md in links
  for f in "$out"/*.md; do
    sed -i \
      -e "s|](Commands\\.md)|](Commands)|g" \
      -e "s|](Architecture\\.md)|](Architecture)|g" \
      -e "s|](Modules\\.md)|](Modules)|g" \
      -e "s|](Conventions\\.md)|](Conventions)|g" \
      -e "s|](Nexus\\.md)|](Nexus)|g" \
      -e "s|](Home\\.md)|](Home)|g" \
      -e "s|](../auth.md)|](${BLOB}/docs/auth.md)|g" \
      -e "s|](../xai-capabilities.md)|](${BLOB}/docs/xai-capabilities.md)|g" \
      -e "s|](../learning.md)|](${BLOB}/docs/learning.md)|g" \
      -e "s|](../missions.md)|](${BLOB}/docs/missions.md)|g" \
      -e "s|](../stress.md)|](${BLOB}/docs/stress.md)|g" \
      -e "s|](../nexus.md)|](Nexus)|g" \
      -e "s|](../architecture.md)|](${BLOB}/docs/architecture.md)|g" \
      -e "s|](../../CHANGELOG.md)|](${BLOB}/CHANGELOG.md)|g" \
      "$f"
  done

  cat > "$out/_Sidebar.md" << SIDE
**Aegis 0.8.0**

- [Home](Home)
- [Commands](Commands)
- [Architecture](Architecture)
- [Modules](Modules)
- [Nexus](Nexus)
- [Conventions](Conventions)

**Repo**
- [Code](https://github.com/${REPO})
- [Releases](https://github.com/${REPO}/releases)
- [Changelog](${BLOB}/CHANGELOG.md)
SIDE
}

WIKI_URL="https://x-access-token:${TOKEN}@github.com/${REPO}.wiki.git"
PREP="$TMP/pages"
prepare "$PREP"

if git ls-remote "$WIKI_URL" HEAD 2>/dev/null | grep -q .; then
  git clone "$WIKI_URL" "$TMP/repo"
  cd "$TMP/repo"
  find . -mindepth 1 -maxdepth 1 ! -name '.git' -exec rm -rf {} +
else
  echo "Wiki git repo empty / missing — bootstrapping first commit…"
  mkdir -p "$TMP/repo"
  cd "$TMP/repo"
  git init -b master
  git remote add origin "$WIKI_URL"
fi

cp -a "$PREP"/. .
git config user.name "aegis-wiki-bot"
git config user.email "aegis-wiki-bot@users.noreply.github.com"
git add -A
if git diff --cached --quiet; then
  echo "Wiki already up to date."
  exit 0
fi
git commit -m "Sync wiki from docs/wiki (Aegis 0.8.0)"
if ! git push -u origin HEAD:master 2>/tmp/wiki-push.err; then
  if ! git push -u origin HEAD:main 2>>/tmp/wiki-push.err; then
    cat /tmp/wiki-push.err >&2
    echo "" >&2
    echo "Bootstrap required (one-time):" >&2
    echo "  1. Open https://github.com/${REPO}/wiki and click 'Create the first page'" >&2
    echo "  2. Save an empty Home page" >&2
    echo "  3. Re-run: ./scripts/publish_wiki.sh" >&2
    echo "Or use a classic PAT with 'repo' scope as GH_TOKEN (fine-grained tokens often cannot access wikis)." >&2
    exit 1
  fi
fi
echo "Published: https://github.com/${REPO}/wiki"
