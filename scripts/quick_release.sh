#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "╔════════════════════════════════════════════════════════════╗"
echo "║         SAS Quick Release                                  ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

echo "Current version: $VERSION"
echo ""
read -p "Enter new version (or press Enter to keep $VERSION): " NEW_VERSION

if [ -n "$NEW_VERSION" ]; then
    echo "→ Updating version to $NEW_VERSION..."
    
    sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
    rm -f Cargo.toml.bak
    
    VERSION="$NEW_VERSION"
    echo "✓ Version updated in Cargo.toml"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Step 1/4: Building releases"
echo "════════════════════════════════════════════════════════════"
echo ""

"$SCRIPT_DIR/create_release.sh"

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Step 2/4: Committing and pushing changes"
echo "════════════════════════════════════════════════════════════"
echo ""

echo "→ Pulling latest changes..."
git pull --rebase origin main
echo "✓ Rebased on origin/main"

git add -A

if git diff --cached --quiet; then
    echo "✓ No changes to commit"
else
    if [ -n "$NEW_VERSION" ]; then
        git commit -m "Bump version to $VERSION"
    else
        git commit -m "Release $VERSION"
    fi
    echo "✓ Changes committed"
fi

echo "→ Pushing to origin/main..."
git push origin main
echo "✓ Changes pushed to GitHub"

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Step 3/4: Publishing to GitHub"
echo "════════════════════════════════════════════════════════════"
echo ""

"$SCRIPT_DIR/publish_release.sh"

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Step 4/4: Cleanup"
echo "════════════════════════════════════════════════════════════"
echo ""

read -p "Delete local release builds? (yes/no): " answer

if [[ "$answer" =~ ^[Yy] ]]; then
    rm -rf release_builds
    echo "✓ Release builds deleted"
else
    echo "✓ Release builds kept in release_builds/"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "  Release Complete! 🚀"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Version $VERSION has been released!"
echo ""



