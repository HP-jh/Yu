//go:build windows

package main

// Windows 侧由宿主注入（PDH），sidecar 只保留占位，避免引入 cgo。
func linuxMetrics(now int64) []metric { return nil }
