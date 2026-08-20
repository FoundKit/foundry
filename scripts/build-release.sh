#!/usr/bin/env bash
# ==============================================================================
# Foundry Unified Release Packaging Pipeline
# Merges Foundry Core Base Infrastructure + Decoupled Custom Subsystems
# ==============================================================================
set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${PROJECT_ROOT}/dist"
OUTPUT_DIR="${DIST_DIR}/release"

echo "======================================================================"
echo "🚀 Starting Foundry Production Packaging Pipeline..."
echo "======================================================================"
echo "📂 Project Root: ${PROJECT_ROOT}"
echo "📦 Output Target: ${OUTPUT_DIR}"

# Parse CLI arguments
CUSTOM_SYSTEMS_DIR="${CUSTOM_SYSTEMS_DIR:-}"
while [[ $# -gt 0 ]]; do
  case $1 in
    --systems-dir)
      CUSTOM_SYSTEMS_DIR="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

if [ -n "${CUSTOM_SYSTEMS_DIR}" ] && [ -d "${CUSTOM_SYSTEMS_DIR}" ]; then
    echo "🔗 Using Custom Subsystems Repository at: ${CUSTOM_SYSTEMS_DIR}"
fi

# 1. Clean previous release artifacts
rm -rf "${OUTPUT_DIR}"
mkdir -p "${OUTPUT_DIR}/bin"
mkdir -p "${OUTPUT_DIR}/static/admin"
mkdir -p "${OUTPUT_DIR}/static/custom_pages"
mkdir -p "${OUTPUT_DIR}/external_systems"
mkdir -p "${OUTPUT_DIR}/migrations"

# 2. Build Frontend Web Admin SPA
echo "📦 [1/4] Building Web Admin Dashboard SPA (React/Vite)..."
if [ -d "${PROJECT_ROOT}/apps/admin" ]; then
    cd "${PROJECT_ROOT}/apps/admin"
    if command -v pnpm &> /dev/null; then
        pnpm install --frozen-lockfile || pnpm install
        pnpm run build
    else
        npm ci || npm install
        npm run build
    fi
    cp -r "${PROJECT_ROOT}/apps/admin/dist/"* "${OUTPUT_DIR}/static/admin/"
    echo "✅ Admin SPA compiled into release bundle."
fi

# 3. Gather Decoupled Subsystems Custom Admin Pages & Manifests
echo "📦 [2/4] Assembling Decoupled Subsystem Custom Admin UI Pages..."
# From compiled systems
if [ -d "${PROJECT_ROOT}/systems/src" ]; then
    for sys_dir in "${PROJECT_ROOT}/systems/src"/*; do
        if [ -d "${sys_dir}/custom_pages" ]; then
            sys_slug=$(basename "${sys_dir}")
            mkdir -p "${OUTPUT_DIR}/static/custom_pages/${sys_slug}"
            cp -r "${sys_dir}/custom_pages/"* "${OUTPUT_DIR}/static/custom_pages/${sys_slug}/"
            echo "   • Bundled compiled custom pages for '${sys_slug}'"
        fi
    done
fi

# From external standalone systems directory in foundry repo
if [ -d "${PROJECT_ROOT}/external_systems" ]; then
    cp -r "${PROJECT_ROOT}/external_systems/"* "${OUTPUT_DIR}/external_systems/"
    echo "   • Bundled standalone external subsystems from 'external_systems/'"
fi

# From dedicated custom subsystems repository (if passed via --systems-dir)
if [ -n "${CUSTOM_SYSTEMS_DIR}" ] && [ -d "${CUSTOM_SYSTEMS_DIR}" ]; then
    echo "   • Bundling subsystems from custom repository: ${CUSTOM_SYSTEMS_DIR}"
    for sys_dir in "${CUSTOM_SYSTEMS_DIR}"/* "${CUSTOM_SYSTEMS_DIR}/systems"/* "${CUSTOM_SYSTEMS_DIR}/src"/*; do
        if [ -d "${sys_dir}" ] && [ -f "${sys_dir}/subsystem.json" ]; then
            sys_slug=$(basename "${sys_dir}")
            mkdir -p "${OUTPUT_DIR}/external_systems/${sys_slug}"
            cp -r "${sys_dir}"/* "${OUTPUT_DIR}/external_systems/${sys_slug}/"
            if [ -d "${sys_dir}/custom_pages" ]; then
                mkdir -p "${OUTPUT_DIR}/static/custom_pages/${sys_slug}"
                cp -r "${sys_dir}/custom_pages/"* "${OUTPUT_DIR}/static/custom_pages/${sys_slug}/"
            fi
            echo "     -> Integrated custom subsystem '${sys_slug}'"
        fi
    done
fi

# 4. Build Release Rust Binaries
echo "📦 [3/4] Compiling Rust Platform Binaries (foundry-server, foundry-cli)..."
cd "${PROJECT_ROOT}"
cargo build --release --bin foundry-server --bin foundry-cli

cp "${PROJECT_ROOT}/target/release/foundry-server" "${OUTPUT_DIR}/bin/"
cp "${PROJECT_ROOT}/target/release/foundry-cli" "${OUTPUT_DIR}/bin/"
echo "✅ Server & CLI release binaries compiled."

# 5. Copy Database Migrations & Scaffolding
echo "📦 [4/4] Bundling Database Migrations and Environment Configuration..."
cp -r "${PROJECT_ROOT}/migrations/"* "${OUTPUT_DIR}/migrations/"
if [ -f "${PROJECT_ROOT}/.env.example" ]; then
    cp "${PROJECT_ROOT}/.env.example" "${OUTPUT_DIR}/.env.example"
fi

cat << 'ENTRYPOINT_EOF' > "${OUTPUT_DIR}/start.sh"
#!/usr/bin/env bash
set -e
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export FOUNDRY_SYSTEMS_DIR="${FOUNDRY_SYSTEMS_DIR:-$DIR/external_systems}"
echo "🚀 Starting Foundry Server from release package..."
exec "$DIR/bin/foundry-server" "$@"
ENTRYPOINT_EOF

chmod +x "${OUTPUT_DIR}/start.sh"
chmod +x "${OUTPUT_DIR}/bin/"*

echo "======================================================================"
echo "🎉 Packaging completed successfully!"
echo "📍 Release Artifact Location: ${OUTPUT_DIR}"
echo "🚀 To run:"
echo "   cd ${OUTPUT_DIR} && ./start.sh"
echo "======================================================================"
