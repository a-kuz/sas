#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
RELEASE_TAG="v$VERSION"
RELEASE_DIR="release_builds"

echo "╔════════════════════════════════════════════════════════════╗"
echo "║         SAS Release Publisher v$VERSION                      "
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

if [ ! -d "$RELEASE_DIR" ] || [ -z "$(ls -A $RELEASE_DIR/*.tar.gz 2>/dev/null)" ]; then
    echo "✗ No release builds found!"
    echo "  Run ./scripts/create_release.sh first"
    exit 1
fi

if ! command -v gh &> /dev/null; then
    echo "✗ GitHub CLI (gh) not found!"
    echo ""
    echo "Install it with:"
    echo "  macOS:   brew install gh"
    echo "  Linux:   See https://github.com/cli/cli#installation"
    echo ""
    exit 1
fi

echo "Checking GitHub authentication..."
if ! gh auth status &> /dev/null; then
    echo "✗ Not authenticated with GitHub!"
    echo ""
    echo "Run: gh auth login"
    echo ""
    exit 1
fi

echo "✓ GitHub CLI authenticated"
echo ""

REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || echo "")

if [ -z "$REPO" ]; then
    echo "✗ Not in a GitHub repository!"
    echo "  Make sure you're in a git repository with a GitHub remote"
    exit 1
fi

echo "Repository: $REPO"
echo "Version: $VERSION"
echo "Tag: $RELEASE_TAG"
echo ""

if git rev-parse "$RELEASE_TAG" >/dev/null 2>&1; then
    echo "⚠ Tag $RELEASE_TAG already exists locally"
    read -p "Delete and recreate? (yes/no): " answer
    
    if [[ "$answer" =~ ^[Yy] ]]; then
        git tag -d "$RELEASE_TAG"
        echo "✓ Local tag deleted"
    else
        echo "✗ Aborted"
        exit 1
    fi
fi

EXISTING_RELEASE=$(gh release view "$RELEASE_TAG" --json tagName -q .tagName 2>/dev/null || echo "")

if [ -n "$EXISTING_RELEASE" ]; then
    echo "⚠ Release $RELEASE_TAG already exists on GitHub"
    read -p "Delete and recreate? (yes/no): " answer
    
    if [[ "$answer" =~ ^[Yy] ]]; then
        gh release delete "$RELEASE_TAG" --yes
        echo "✓ Remote release deleted"
    else
        echo "✗ Aborted"
        exit 1
    fi
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Creating Git Tag"
echo "════════════════════════════════════════════════════════════"
echo ""

git tag -a "$RELEASE_TAG" -m "Release $VERSION"
echo "✓ Tag $RELEASE_TAG created"

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Generating Release Notes"
echo "════════════════════════════════════════════════════════════"
echo ""

RELEASE_NOTES="Release $VERSION

## Installation

### Quick Install (Recommended)

**macOS/Linux:**
\`\`\`bash
curl -fsSL https://raw.githubusercontent.com/$REPO/main/scripts/install.sh | bash
\`\`\`

**Windows:**
\`\`\`cmd
curl -fsSL https://raw.githubusercontent.com/$REPO/main/scripts/install.bat -o install.bat && install.bat
\`\`\`

### Manual Install

1. Download the appropriate archive for your platform
2. Extract the archive
3. Run the \`sas\` executable (or \`sas.exe\` on Windows)

**Note:** You need to own a legal copy of Quake 3 Arena. The installer will download required game resources.

## What's Included

- Game binary optimized for your platform
- Assets and maps
- Configuration files

## System Requirements

- Quake 3 Arena (legal copy)
- OpenGL 3.3+ compatible graphics
- 2GB RAM minimum
- 500MB disk space

## Platforms

- macOS (Intel & Apple Silicon)
- Linux (x86_64)
- Windows (x86_64)

---

**Built with Rust 🦀**
"

NOTES_FILE="$RELEASE_DIR/release_notes.md"
echo "$RELEASE_NOTES" > "$NOTES_FILE"

echo "✓ Release notes generated"

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Publishing Release"
echo "════════════════════════════════════════════════════════════"
echo ""

echo "→ Pushing tag to GitHub..."
git push origin "$RELEASE_TAG"

echo "→ Creating GitHub release..."

ASSETS=""
for archive in "$RELEASE_DIR"/*.tar.gz; do
    ASSETS="$ASSETS $archive"
done

gh release create "$RELEASE_TAG" \
    --title "SAS v$VERSION" \
    --notes-file "$NOTES_FILE" \
    $ASSETS

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Release Published Successfully! 🎉"
echo "════════════════════════════════════════════════════════════"
echo ""

RELEASE_URL=$(gh release view "$RELEASE_TAG" --json url -q .url)

echo "Release URL: $RELEASE_URL"
echo ""
echo "Uploaded files:"
ls -lh "$RELEASE_DIR"/*.tar.gz | awk '{print "  " $9 " (" $5 ")"}'
echo ""
echo "Users can now install with:"
echo "  curl -fsSL https://raw.githubusercontent.com/$REPO/main/scripts/install.sh | bash"
echo ""






