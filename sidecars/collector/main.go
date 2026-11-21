// 屿 · 采集 sidecar（Go，零依赖）
//
// 三条通道：
//   1) stdio JSON-RPC：宿主拉起后按行请求 {"id","method","params"}
//   2) 本地 HTTP 127.0.0.1:39272：/health /metrics /mcp
//   3) 只读 ~/.yu，不外发任何内容
package main

import (
	"bufio"
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"net/http"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"time"
)

type metric struct {
	Kind  string  `json:"kind"`
	Value float64 `json:"value"`
	At    int64   `json:"at_ms"`
}

type rpcReq struct {
	ID     any             `json:"id"`
	Method string          `json:"method"`
	Params json.RawMessage `json:"params"`
}

type rpcResp struct {
	ID     any    `json:"id"`
	Result any    `json:"result,omitempty"`
	Error  string `json:"error,omitempty"`
}

func main() {
	httpAddr := flag.String("http", "", "HTTP 监听地址（如 127.0.0.1:39272）")
	stdio := flag.Bool("rpc", false, "stdio JSON-RPC 模式")
	flag.Parse()

	if *stdio {
		serveStdio()
		return
	}

	addr := *httpAddr
	if addr == "" {
		addr = "127.0.0.1:39272"
	}
	mux := http.NewServeMux()
	mux.HandleFunc("/health", func(w http.ResponseWriter, _ *http.Request) {
		writeJSON(w, map[string]any{"ok": true, "os": runtime.GOOS, "ts": time.Now().UnixMilli()})
	})
	mux.HandleFunc("/metrics", func(w http.ResponseWriter, _ *http.Request) {
		writeJSON(w, collect())
	})
	mux.HandleFunc("/mcp", handleMCP)
	log.Printf("collector on http://%s", addr)
	log.Fatal(http.ListenAndServe(addr, mux))
}

func serveStdio() {
	sc := bufio.NewScanner(os.Stdin)
	enc := json.NewEncoder(os.Stdout)
	for sc.Scan() {
		line := strings.TrimSpace(sc.Text())
		if line == "" {
			continue
		}
		var req rpcReq
		if err := json.Unmarshal([]byte(line), &req); err != nil {
			_ = enc.Encode(rpcResp{Error: "bad json"})
			continue
		}
		_ = enc.Encode(rpcResp{ID: req.ID, Result: dispatch(req)})
	}
}

func dispatch(req rpcReq) any {
	switch req.Method {
	case "metrics":
		return collect()
	case "state":
		return map[string]any{"mode": "idle", "occupation": "student"}
	case "todos":
		return map[string]any{"pending": countTodos()}
	default:
		return map[string]any{"error": "unknown method"}
	}
}

func handleMCP(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		w.WriteHeader(http.StatusMethodNotAllowed)
		return
	}
	var req struct {
		Method string          `json:"method"`
		Params json.RawMessage `json:"params"`
	}
	_ = json.NewDecoder(r.Body).Decode(&req)
	switch req.Method {
	case "tools/list":
		writeJSON(w, map[string]any{"tools": []map[string]any{
			{"name": "yu.state", "description": "岛当前形态与职业"},
			{"name": "yu.todos", "description": "今日待办（只读）"},
			{"name": "yu.notify", "description": "发一条通知（受打扰预算限制）"},
		}})
	case "tools/call":
		writeJSON(w, map[string]any{"ok": true, "note": "v0.1 只读占位"})
	default:
		writeJSON(w, map[string]any{"ok": false, "error": "unknown"})
	}
}

func writeJSON(w http.ResponseWriter, v any) {
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	_ = json.NewEncoder(w).Encode(v)
}

func collect() []metric {
	now := time.Now().UnixMilli()
	out := []metric{}
	if runtime.GOOS == "linux" {
		out = append(out, linuxMetrics(now)...)
	}
	if len(out) == 0 {
		out = append(out, metric{Kind: "unsupported", Value: 0, At: now})
	}
	return out
}

func yuHome() string {
	home, _ := os.UserHomeDir()
	return filepath.Join(home, ".yu")
}

func countTodos() int {
	raw, err := os.ReadFile(filepath.Join(yuHome(), "todos.json"))
	if err != nil {
		return -1
	}
	var v struct {
		Todos []struct {
			Done bool `json:"done"`
		} `json:"todos"`
	}
	if json.Unmarshal(raw, &v) != nil {
		return -1
	}
	n := 0
	for _, t := range v.Todos {
		if !t.Done {
			n++
		}
	}
	return n
}

func debugf(f string, a ...any) {
	if os.Getenv("YU_DEBUG") != "" {
		fmt.Fprintf(os.Stderr, f+"\n", a...)
	}
}
