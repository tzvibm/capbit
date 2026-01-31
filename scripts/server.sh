#!/bin/bash
# Capbit server management script
# Works with Claude CLI in Termux

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
SERVER_BIN="$PROJECT_DIR/target/debug/capbit-server"
LOG_FILE="$PROJECT_DIR/capbit-server.log"
PID_FILE="$PROJECT_DIR/capbit-server.pid"
PORT="${PORT:-3000}"

build() {
    echo "Building server..."
    cd "$PROJECT_DIR"
    cargo build --features server --bin capbit-server
}

start() {
    if [ ! -f "$SERVER_BIN" ]; then
        echo "Server binary not found. Building..."
        build
    fi

    if pgrep -f "capbit-server" > /dev/null 2>&1; then
        echo "Server already running. Stop it first with: $0 stop"
        exit 1
    fi

    echo "Starting Capbit server on port $PORT..."
    cd "$PROJECT_DIR"
    PORT=$PORT setsid "$SERVER_BIN" > "$LOG_FILE" 2>&1 &
    echo $! > "$PID_FILE"

    # Wait for server to be ready
    for i in 1 2 3 4 5; do
        sleep 1
        if curl -s "http://localhost:$PORT/health" > /dev/null 2>&1; then
            echo "Server running at http://localhost:$PORT"
            echo "Logs: $LOG_FILE"
            exit 0
        fi
    done

    echo "Server failed to start. Check logs: $LOG_FILE"
    cat "$LOG_FILE"
    exit 1
}

stop() {
    echo "Stopping Capbit server..."
    pkill -f "capbit-server" 2>/dev/null || true
    rm -f "$PID_FILE"
    echo "Server stopped."
}

status() {
    if curl -s "http://localhost:$PORT/health" > /dev/null 2>&1; then
        echo "Server is running on port $PORT"
        curl -s "http://localhost:$PORT/status" | python3 -m json.tool 2>/dev/null || curl -s "http://localhost:$PORT/status"
    else
        echo "Server is not running"
    fi
}

logs() {
    if [ -f "$LOG_FILE" ]; then
        cat "$LOG_FILE"
    else
        echo "No log file found"
    fi
}

case "${1:-}" in
    start)   start ;;
    stop)    stop ;;
    restart) stop; start ;;
    status)  status ;;
    logs)    logs ;;
    build)   build ;;
    *)
        echo "Capbit Server Management"
        echo ""
        echo "Usage: $0 {start|stop|restart|status|logs|build}"
        echo ""
        echo "Commands:"
        echo "  start   - Start the server (builds if needed)"
        echo "  stop    - Stop the server"
        echo "  restart - Restart the server"
        echo "  status  - Check if server is running"
        echo "  logs    - Show server logs"
        echo "  build   - Build the server binary"
        echo ""
        echo "Environment:"
        echo "  PORT    - Server port (default: 3000)"
        echo ""
        echo "Example:"
        echo "  PORT=3001 $0 start"
        ;;
esac
