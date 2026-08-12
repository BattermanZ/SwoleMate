# Canonical local development-server lifecycle. Agents must use these recipes;
# see AGENTS.md.

cargo_target_dir := "/scratch/cargo-target"
cargo_home := "/scratch/cargo-home"
backend_port := "2469"
frontend_port := "2470"
dev_dir := ".dev"
target_warn_gb := "20"

export CARGO_TARGET_DIR := cargo_target_dir
export CARGO_HOME := cargo_home

default:
    @just --list

# Start the Rust API and Vite frontend with hot reload. Safe to re-run: any
# tracked or stale processes on the development ports are stopped first.
dev-start: _kill-stale
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p {{dev_dir}}

    if [ -d "$CARGO_TARGET_DIR" ]; then
        size_kb=$(du -sk "$CARGO_TARGET_DIR" 2>/dev/null | cut -f1)
        size_gb=$(( size_kb / 1024 / 1024 ))
        if [ "$size_gb" -ge {{target_warn_gb}} ]; then
            echo "warning: $CARGO_TARGET_DIR is ${size_gb}G (>= {{target_warn_gb}}G) - run 'just dev-clean' to reclaim space" >&2
        fi
    fi

    echo "starting backend (cargo run)..."
    cd server
    setsid cargo run > ../{{dev_dir}}/backend.log 2>&1 &
    echo $! > ../{{dev_dir}}/backend.pid
    cd ..

    echo "starting frontend (npm run dev)..."
    cd client
    setsid npm run dev -- --host 0.0.0.0 --port {{frontend_port}} --strictPort > ../{{dev_dir}}/frontend.log 2>&1 &
    echo $! > ../{{dev_dir}}/frontend.pid
    cd ..

    sleep 1
    echo
    echo "backend log:  {{dev_dir}}/backend.log   (http://127.0.0.1:{{backend_port}}, compiling may take a moment)"
    echo "frontend log: {{dev_dir}}/frontend.log  (http://localhost:{{frontend_port}})"
    echo "'just dev-status' to check, 'just dev-stop' to stop"

# Stop the tracked backend/frontend process groups and stale port listeners.
dev-stop: _kill-stale
    @echo "stopped"

_kill-stale:
    #!/usr/bin/env bash
    set -uo pipefail
    for name in backend frontend; do
        pidfile="{{dev_dir}}/${name}.pid"
        if [ -f "$pidfile" ]; then
            pid=$(cat "$pidfile")
            if kill -0 "$pid" 2>/dev/null; then
                echo "stopping tracked ${name} (pid $pid)"
                kill -s TERM -- "-$pid" 2>/dev/null || kill -s TERM "$pid" 2>/dev/null || true
                sleep 1
                kill -0 "$pid" 2>/dev/null && { kill -s KILL -- "-$pid" 2>/dev/null || kill -s KILL "$pid" 2>/dev/null || true; }
            fi
            rm -f "$pidfile"
        fi
    done
    fuser -k -TERM {{backend_port}}/tcp 2>/dev/null || true
    fuser -k -TERM {{frontend_port}}/tcp 2>/dev/null || true
    sleep 1
    true

# Report tracked process state and Cargo build-cache size.
dev-status:
    #!/usr/bin/env bash
    set -uo pipefail
    for name in backend frontend; do
        pidfile="{{dev_dir}}/${name}.pid"
        if [ -f "$pidfile" ] && kill -0 "$(cat "$pidfile")" 2>/dev/null; then
            pid=$(cat "$pidfile")
            started=$(ps -o lstart= -p "$pid" 2>/dev/null | xargs)
            echo "${name}: running (pid $pid, started $started)"
        else
            echo "${name}: not running"
        fi
    done
    if [ -d "$CARGO_TARGET_DIR" ]; then
        echo "cargo target dir ($CARGO_TARGET_DIR): $(du -sh "$CARGO_TARGET_DIR" 2>/dev/null | cut -f1)"
    fi

# Reclaim the shared Cargo build cache. The next build will be a full rebuild.
dev-clean:
    cd server && cargo clean

# Build the client and run both production-mode processes in the foreground.
# Ctrl+C stops both; this does not build or start containers.
prod-check: _kill-stale
    #!/usr/bin/env bash
    set -euo pipefail
    echo "building frontend..."
    (cd client && npm run build)
    trap 'kill 0' EXIT INT TERM
    (cd server && cargo run) &
    (cd client && npm run preview -- --host 0.0.0.0 --port {{frontend_port}} --strictPort) &
    wait
