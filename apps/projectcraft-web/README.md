# ProjectCraft Web (M5 first browser slice)

This is the **actual ProjectCraft egui UI**, not an HTML imitation or an
embedded report-rs view. The same \`projectcraft-ui-egui::App\` and
\`projectcraft-engine::Session\` are used on desktop and web.

## Scope and safety

The initial Railway instance starts with ProjectCraft's synthetic sample.
Changes are stored in the visitor's browser localStorage; they are NOT shared
with other users or written to a backend. Never treat this as customer
collaboration or a published, authorized project view.

Native Open/Save/Export requests are intentionally cancelled in the browser
rather than pointing to nonexistent filesystem paths. The next development
step is proper browser file import/export followed by a separately gated
OIDC + project-command + PostgreSQL service.

## Build

From the repository root:

\`\`\`sh
rustup target add wasm32-unknown-unknown
cargo install trunk --version 0.22.1 --locked
cd apps/projectcraft-web
trunk build --release --public-url /
cd ../..
cargo build --release -p projectcraft-web --features web-host --bin projectcraft-web
PORT=8080 target/release/projectcraft-web
\`\`\`

Open http://localhost:8080, and GET /healthz returns \`ok\`.

On Railway, use \`Dockerfile.web\` and a separate service under the existing
\`report-rs\` project. The service must NOT inherit the portal's production
credentials, GRAPH permissions, or ZITADEL secrets.
