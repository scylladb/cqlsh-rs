#!/usr/bin/env python3
"""
Generate an SVG roadmap/progress infographic for cqlsh-rs.

Reads docs/progress.json and produces docs/assets/progress-roadmap.svg.

GitHub-compatible: no <animate>, no <filter>, no emoji in <text>,
no feDropShadow/feGaussianBlur. Uses only basic SVG elements that
survive GitHub's SVG sanitizer (camo.githubusercontent.com).
"""

import json
import sys
from datetime import datetime, timedelta
from html import escape as xml_escape
from pathlib import Path


def load_progress(path: str) -> dict:
    with open(path) as f:
        return json.load(f)


def compute_velocity(data: dict) -> dict:
    """Compute tasks/day velocity from the merged PR log."""
    velocity = data.get("velocity", {})
    log = velocity.get("tasks_completed_log", [])
    started = velocity.get("started_date")

    if not log or not started:
        return {"tasks_per_day": 0, "eta_text": "Calculating...", "eta_silly": "Discovering remaining items..."}

    start_dt = datetime.strptime(started, "%Y-%m-%d")
    now = datetime.now()
    elapsed_days = max((now - start_dt).days, 1)

    total_done_from_log = sum(entry["tasks_done"] for entry in log)
    tasks_per_day = total_done_from_log / elapsed_days

    total_tasks = sum(p["total_tasks"] for p in data["phases"])
    completed_tasks = sum(p["completed_tasks"] for p in data["phases"])
    remaining = total_tasks - completed_tasks

    if tasks_per_day > 0:
        days_remaining = remaining / tasks_per_day
        eta_date = now + timedelta(days=days_remaining)
        eta_text = f"~{int(days_remaining)} days remaining (ETA: {eta_date.strftime('%b %Y')})"

        if days_remaining < 1:
            eta_silly = "Less than a day remaining... probably"
        elif days_remaining < 7:
            hours = int(days_remaining * 24)
            eta_silly = f"About {hours} hours remaining... give or take a mass rebuild"
        elif days_remaining < 30:
            eta_silly = f"About {int(days_remaining)} days remaining... {int(remaining)} features to copy to Rust"
        elif days_remaining < 365:
            months = days_remaining / 30
            eta_silly = f"Approximately {months:.1f} months remaining... just like Windows said"
        else:
            years = days_remaining / 365
            eta_silly = f"About {years:.1f} years remaining... have you tried turning it off and on again?"
    else:
        eta_text = "No velocity data yet"
        eta_silly = "Discovering remaining items..."
        days_remaining = float("inf")

    return {
        "tasks_per_day": tasks_per_day,
        "days_remaining": days_remaining,
        "eta_text": eta_text,
        "eta_silly": eta_silly,
        "velocity_label": f"{tasks_per_day:.1f} tasks/day",
    }


def phase_color(status: str) -> str:
    if status == "completed":
        return "#22c55e"
    elif status == "in_progress":
        return "#f59e0b"
    else:
        return "#64748b"


def phase_color_dim(status: str) -> str:
    if status == "completed":
        return "#166534"
    elif status == "in_progress":
        return "#92400e"
    else:
        return "#334155"


def generate_svg(data: dict) -> str:
    phases = data["phases"]
    total_tasks = sum(p["total_tasks"] for p in phases)
    completed_tasks = sum(p["completed_tasks"] for p in phases)
    overall_pct = (completed_tasks / total_tasks * 100) if total_tasks > 0 else 0

    vel = compute_velocity(data)

    # Layout
    W = 880
    H = 540
    margin_x = 40
    road_y = 195
    R = 24
    gap = (W - 2 * margin_x) / len(phases)

    s = []
    a = s.append

    a(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}">')

    # Defs: gradients, clipPath, and CSS animations
    # CSS animations work when SVG is viewed directly (clicking through on GitHub).
    # They are stripped when rendered via <img> tags (browser security), but the
    # static fallback still looks good.
    a("""  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#0f172a"/>
      <stop offset="100%" stop-color="#1e293b"/>
    </linearGradient>
    <linearGradient id="barFill" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0%" stop-color="#22c55e"/>
      <stop offset="100%" stop-color="#4ade80"/>
    </linearGradient>
    <linearGradient id="barActive" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0%" stop-color="#f59e0b"/>
      <stop offset="100%" stop-color="#fbbf24"/>
    </linearGradient>
    <clipPath id="barClip">
      <rect x="90" y="383" width="700" height="20" rx="3"/>
    </clipPath>
    <style>
      @keyframes pulse {
        0%, 100% { r: 28; opacity: 0.4; }
        50% { r: 34; opacity: 0.15; }
      }
      @keyframes pulse-inner {
        0%, 100% { r: 32; opacity: 0.25; }
        50% { r: 36; opacity: 0.1; }
      }
      .pulse-ring { animation: pulse 2s ease-in-out infinite; }
      .pulse-ring-outer { animation: pulse-inner 2s ease-in-out infinite; }
      @keyframes glow-text {
        0%, 100% { opacity: 1; }
        50% { opacity: 0.6; }
      }
      .glow { animation: glow-text 2s ease-in-out infinite; }
    </style>
  </defs>""")

    # Background
    a(f'  <rect width="{W}" height="{H}" fill="url(#bg)" rx="12"/>')

    # Decorative border
    a(f'  <rect x="1" y="1" width="{W-2}" height="{H-2}" fill="none" stroke="#1e293b" stroke-width="2" rx="12"/>')

    # ── Title ──
    a(f'  <text x="{W//2}" y="40" text-anchor="middle" font-family="monospace" font-size="20" font-weight="700" fill="#f8fafc">cqlsh-rs Development Roadmap</text>')
    a(f'  <text x="{W//2}" y="62" text-anchor="middle" font-family="sans-serif" font-size="12" fill="#94a3b8">Rewriting cqlsh in Rust -- one semicolon at a time</text>')

    # ── Stats line ──
    stats_left = f"{completed_tasks}/{total_tasks} tasks ({overall_pct:.0f}%)"
    a(f'  <text x="260" y="88" text-anchor="middle" font-family="monospace" font-size="13" fill="#e2e8f0">{stats_left}</text>')
    if vel["tasks_per_day"] > 0:
        stats_right = f"{vel['velocity_label']}  |  {vel['eta_text']}"
        a(f'  <text x="600" y="88" text-anchor="middle" font-family="monospace" font-size="12" fill="#94a3b8">{stats_right}</text>')

    # ── Separator line ──
    a(f'  <line x1="60" y1="100" x2="{W-60}" y2="100" stroke="#1e293b" stroke-width="1"/>')

    # ── Road/track ──
    rx0 = margin_x + gap * 0.5
    rx1 = margin_x + gap * (len(phases) - 0.5)
    # Road background
    a(f'  <line x1="{rx0}" y1="{road_y}" x2="{rx1}" y2="{road_y}" stroke="#334155" stroke-width="8" stroke-linecap="round"/>')

    # Colored road overlays for completed/in-progress
    for i, phase in enumerate(phases):
        cx = margin_x + gap * (i + 0.5)
        if i > 0:
            prev_cx = margin_x + gap * (i - 0.5)
            prev = phases[i - 1]
            if prev["status"] == "completed":
                a(f'  <line x1="{prev_cx}" y1="{road_y}" x2="{cx}" y2="{road_y}" stroke="#22c55e" stroke-width="8" stroke-linecap="round" opacity="0.6"/>')
            elif prev["status"] == "in_progress":
                pct_prev = (prev["completed_tasks"] / prev["total_tasks"]) if prev["total_tasks"] > 0 else 0
                partial_x = prev_cx + (cx - prev_cx) * pct_prev
                a(f'  <line x1="{prev_cx}" y1="{road_y}" x2="{partial_x}" y2="{road_y}" stroke="#f59e0b" stroke-width="8" stroke-linecap="round" opacity="0.6"/>')

    # ── Phase nodes ──
    you_are_here_cx = None
    for i, phase in enumerate(phases):
        cx = margin_x + gap * (i + 0.5)
        pct = (phase["completed_tasks"] / phase["total_tasks"] * 100) if phase["total_tasks"] > 0 else 0
        fill = phase_color(phase["status"])
        dim = phase_color_dim(phase["status"])

        if phase["status"] == "in_progress":
            you_are_here_cx = cx
            # Animated pulse rings (CSS animation; static fallback when in <img>)
            a(f'  <circle class="pulse-ring-outer" cx="{cx}" cy="{road_y}" r="{R + 8}" fill="none" stroke="{fill}" stroke-width="2" opacity="0.25"/>')
            a(f'  <circle class="pulse-ring" cx="{cx}" cy="{road_y}" r="{R + 4}" fill="none" stroke="{fill}" stroke-width="1.5" opacity="0.4"/>')

        # Node: outer ring + filled circle
        a(f'  <circle cx="{cx}" cy="{road_y}" r="{R}" fill="{dim}" stroke="{fill}" stroke-width="3"/>')

        # Phase number
        a(f'  <text x="{cx}" y="{road_y + 6}" text-anchor="middle" font-family="monospace" font-size="16" font-weight="700" fill="#fff">{phase["id"]}</text>')

        # Phase name (above) — escape for XML safety
        a(f'  <text x="{cx}" y="{road_y - 40}" text-anchor="middle" font-family="sans-serif" font-size="11" font-weight="600" fill="#e2e8f0">{xml_escape(phase["name"])}</text>')

        # Task count (below)
        a(f'  <text x="{cx}" y="{road_y + 48}" text-anchor="middle" font-family="monospace" font-size="10" fill="#94a3b8">{phase["completed_tasks"]}/{phase["total_tasks"]} tasks</text>')

        # Percentage
        if pct > 0:
            a(f'  <text x="{cx}" y="{road_y + 62}" text-anchor="middle" font-family="monospace" font-size="10" font-weight="700" fill="{fill}">{pct:.0f}%</text>')

        # Checkmark for completed (using a circle+path instead of emoji)
        if phase["status"] == "completed":
            ck_y = road_y - 56
            a(f'  <circle cx="{cx}" cy="{ck_y}" r="8" fill="#22c55e"/>')
            a(f'  <path d="M{cx-4} {ck_y} l3 3 l5 -6" fill="none" stroke="#fff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>')

        # Description
        a(f'  <text x="{cx}" y="{road_y + 78}" text-anchor="middle" font-family="sans-serif" font-size="8" fill="#64748b">{xml_escape(phase["description"])}</text>')

    # ── "You Are Here" indicator (arrow + text, no emoji) ──
    if you_are_here_cx:
        arr_y = road_y - 72
        # Downward pointing triangle
        x1, x2, x3 = you_are_here_cx - 6, you_are_here_cx + 6, you_are_here_cx
        y1, y2, y3 = arr_y, arr_y, arr_y + 8
        a(f'  <polygon points="{x1},{y1} {x2},{y2} {x3},{y3}" fill="#fbbf24"/>')
        a(f'  <text class="glow" x="{you_are_here_cx}" y="{arr_y - 4}" text-anchor="middle" font-family="sans-serif" font-size="9" font-weight="700" fill="#fbbf24">YOU ARE HERE</text>')

    # ── Legend ──
    leg_y = 315
    for j, (color, label) in enumerate([("#22c55e", "Completed"), ("#f59e0b", "In Progress"), ("#64748b", "Upcoming")]):
        lx = 300 + j * 120
        a(f'  <circle cx="{lx}" cy="{leg_y}" r="5" fill="{color}"/>')
        a(f'  <text x="{lx + 10}" y="{leg_y + 4}" font-family="sans-serif" font-size="11" fill="#cbd5e1">{label}</text>')

    # ══════════════════════════════════════════════════════
    # WINDOWS PROGRESS BAR (velocity-based ETA)
    # ══════════════════════════════════════════════════════
    bar_x = 90
    bar_w = 700
    bar_h = 20
    win_y = 340

    # Window frame
    a(f'  <rect x="{bar_x - 10}" y="{win_y}" width="{bar_w + 20}" height="108" rx="6" fill="#1e293b" stroke="#475569" stroke-width="1"/>')

    # Title bar
    a(f'  <rect x="{bar_x - 10}" y="{win_y}" width="{bar_w + 20}" height="24" rx="6" fill="#334155"/>')
    a(f'  <rect x="{bar_x - 10}" y="{win_y + 18}" width="{bar_w + 20}" height="6" fill="#334155"/>')  # square off bottom corners of title
    a(f'  <text x="{bar_x + 4}" y="{win_y + 16}" font-family="sans-serif" font-size="11" fill="#94a3b8">Copying {total_tasks - completed_tasks} features to Rust...</text>')

    # Window buttons (macOS style dots)
    for k, c in enumerate(["#ef4444", "#f59e0b", "#22c55e"]):
        bx = bar_x + bar_w - 8 - k * 16
        a(f'  <circle cx="{bx}" cy="{win_y + 12}" r="5" fill="{c}" opacity="0.6"/>')

    # Progress bar track
    a(f'  <rect x="{bar_x}" y="383" width="{bar_w}" height="{bar_h}" rx="3" fill="#0f172a" stroke="#475569" stroke-width="1"/>')

    # Chunky segmented blocks (Windows XP style)
    filled_w = bar_w * (overall_pct / 100)
    block_w = 12
    block_gap = 2
    num_blocks = max(int(filled_w / (block_w + block_gap)), 0)

    a('  <g clip-path="url(#barClip)">')
    for b in range(num_blocks):
        bx = bar_x + 2 + b * (block_w + block_gap)
        by = 385
        bh = bar_h - 4
        # Last 2 blocks amber if any phase is in-progress
        is_active = any(p["status"] == "in_progress" for p in phases) and b >= max(num_blocks - 2, 0)
        fill = "url(#barActive)" if is_active else "url(#barFill)"
        a(f'    <rect x="{bx}" y="{by}" width="{block_w}" height="{bh}" rx="2" fill="{fill}" opacity="0.9"/>')
    a('  </g>')

    # Percentage text on the bar
    a(f'  <text x="{bar_x + bar_w / 2}" y="397" text-anchor="middle" font-family="monospace" font-size="11" font-weight="700" fill="#fff" opacity="0.9">{overall_pct:.0f}%</text>')

    # Silly Windows estimate (velocity-based)
    a(f'  <text x="{bar_x + bar_w / 2}" y="425" text-anchor="middle" font-family="sans-serif" font-size="10" fill="#64748b" font-style="italic">{xml_escape(vel["eta_silly"])}</text>')

    # Velocity stats
    merged_prs = data.get("velocity", {}).get("merged_prs", 0)
    if merged_prs > 0:
        stats = f"{merged_prs} PRs merged  |  {vel['velocity_label']}  |  {completed_tasks} tasks done"
        a(f'  <text x="{bar_x + bar_w / 2}" y="442" text-anchor="middle" font-family="monospace" font-size="8" fill="#475569">{stats}</text>')

    # ── Footer ──
    a(f'  <text x="{W // 2}" y="{H - 14}" text-anchor="middle" font-family="sans-serif" font-size="9" fill="#334155">Auto-generated from docs/progress.json  |  Last updated: {data["last_updated"]}</text>')

    a('</svg>')
    return '\n'.join(s)


def main():
    repo_root = Path(__file__).resolve().parent.parent
    progress_path = repo_root / "docs" / "progress.json"
    output_path = repo_root / "docs" / "assets" / "progress-roadmap.svg"

    if not progress_path.exists():
        print(f"Error: {progress_path} not found", file=sys.stderr)
        sys.exit(1)

    data = load_progress(str(progress_path))
    svg = generate_svg(data)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w") as f:
        f.write(svg)

    total = sum(p["total_tasks"] for p in data["phases"])
    done = sum(p["completed_tasks"] for p in data["phases"])
    print(f"Generated {output_path}")
    print(f"  Progress: {done}/{total} tasks ({done / total * 100:.0f}%)")


if __name__ == "__main__":
    main()
