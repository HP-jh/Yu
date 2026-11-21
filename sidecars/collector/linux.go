//go:build linux

package main

import (
	"os"
	"strconv"
	"strings"
)

func linuxMetrics(now int64) []metric {
	out := []metric{}

	raw, err := os.ReadFile("/proc/stat")
	if err == nil {
		first := strings.SplitN(string(raw), "\n", 2)[0]
		if strings.HasPrefix(first, "cpu ") {
			fields := strings.Fields(first)[1:]
			var total, idle float64
			for i, s := range fields {
				v, _ := strconv.ParseFloat(s, 64)
				if i < 8 {
					total += v
				}
				if i == 3 || i == 4 {
					idle += v
				}
			}
			if total > 0 {
				out = append(out, metric{Kind: "cpu_percent", Value: (total - idle) / total * 100, At: now})
			}
		}
	}

	mem, err := os.ReadFile("/proc/meminfo")
	if err == nil {
		var total, avail float64
		for _, line := range strings.Split(string(mem), "\n") {
			if strings.HasPrefix(line, "MemTotal:") {
				total, _ = strconv.ParseFloat(strings.Fields(line)[1], 64)
			}
			if strings.HasPrefix(line, "MemAvailable:") {
				avail, _ = strconv.ParseFloat(strings.Fields(line)[1], 64)
			}
		}
		if total > 0 {
			used := total - avail
			out = append(out, metric{Kind: "mem_percent", Value: used / total * 100, At: now})
			out = append(out, metric{Kind: "mem_used_mb", Value: used / 1024, At: now})
		}
	}

	bat, err := os.ReadFile("/sys/class/power_supply/BAT0/capacity")
	if err == nil {
		if v, e := strconv.ParseFloat(strings.TrimSpace(string(bat)), 64); e == nil {
			out = append(out, metric{Kind: "battery_percent", Value: v, At: now})
		}
	}
	return out
}
