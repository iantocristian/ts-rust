//! Repository tasks. `cargo xtask status` computes progress from evidence:
//! the ledger (PORTS.toml), `port:` markers in Rust sources against the Go function
//! inventory, the experiments file, the ADRs, and the sprint files. It writes STATUS.md,
//! status/status.json and docs/status.html, and with --record appends to status/history.jsonl.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

// ---------------------------------------------------------------- inputs

#[derive(Deserialize)]
struct Ledger {
    #[serde(default)]
    pin: String,
    #[serde(default)]
    file: Vec<LedgerFile>,
}

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
struct LedgerFile {
    go: String,
    package: String,
    #[serde(rename = "crate")]
    krate: String,
    phase: i64,
    kind: String,
    status: String,
    #[serde(default)]
    rust: String,
    #[serde(default)]
    pin: String,
    #[serde(default)]
    loc: i64,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Experiment {
    title: String,
    #[serde(default)]
    measures: String,
    op: String,
    threshold: f64,
    #[serde(default)]
    unit: String,
    #[serde(default)]
    nature: String,
    #[serde(default)]
    measured: Option<f64>,
    #[serde(default)]
    measured_at: String,
    #[serde(default)]
    note: String,
}

#[derive(Deserialize)]
struct Sprint {
    id: String,
    title: String,
    #[serde(default)]
    goal: String,
    #[serde(default)]
    exit: Vec<String>,
    #[serde(default)]
    item: Vec<SprintItem>,
}

#[derive(Deserialize)]
struct SprintItem {
    id: String,
    title: String,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    r#ref: String,
    #[serde(default)]
    done_when: Vec<String>,
}

// ---------------------------------------------------------------- metrics

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
enum Metric {
    Num(f64),
    Str(String),
}

type Metrics = BTreeMap<String, Metric>;

fn status_rank(s: &str) -> f64 {
    match s {
        "planned" => 0.0,
        "in-progress" => 1.0,
        "ported" => 2.0,
        "verified" => 3.0,
        _ => -1.0,
    }
}

fn ratio(n: f64, d: f64) -> f64 {
    if d == 0.0 {
        0.0
    } else {
        n / d
    }
}

struct Report {
    metrics: Metrics,
    ledger_pin: String,
    files_in_scope: Vec<LedgerFile>,
    unported_by_package: BTreeMap<String, Vec<String>>,
    unknown_markers: Vec<String>,
    experiments: Vec<(String, Experiment, Option<bool>)>,
    sprints: Vec<SprintResult>,
    adrs: Vec<(String, String, String)>,
}

struct SprintResult {
    id: String,
    title: String,
    goal: String,
    done: bool,
    exit: Vec<(String, Option<bool>)>,
    items: Vec<(String, String, Option<bool>)>,
}

// ---------------------------------------------------------------- collection

fn read_ledger(root: &Path) -> Ledger {
    let p = root.join("PORTS.toml");
    let text = fs::read_to_string(&p).unwrap_or_else(|e| die(&format!("{}: {e}", p.display())));
    toml::from_str(&text).unwrap_or_else(|e| die(&format!("PORTS.toml: {e}")))
}

fn read_experiments(root: &Path) -> BTreeMap<String, Experiment> {
    let p = root.join("status/experiments.toml");
    match fs::read_to_string(&p) {
        Ok(text) => toml::from_str(&text).unwrap_or_else(|e| die(&format!("experiments.toml: {e}"))),
        Err(_) => BTreeMap::new(),
    }
}

fn read_sprints(root: &Path) -> Vec<Sprint> {
    let dir = root.join("sprints");
    let mut out = Vec::new();
    if let Ok(rd) = fs::read_dir(&dir) {
        let mut paths: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.extension().map(|x| x == "toml").unwrap_or(false)).collect();
        paths.sort();
        for p in paths {
            let text = fs::read_to_string(&p).unwrap_or_default();
            match toml::from_str::<Sprint>(&text) {
                Ok(s) => out.push(s),
                Err(e) => die(&format!("{}: {e}", p.display())),
            }
        }
    }
    out
}

/// ADRs: (number, title, status) from docs/adr/NNNN-*.md files.
fn read_adrs(root: &Path) -> Vec<(String, String, String)> {
    let dir = root.join("docs/adr");
    let mut out = Vec::new();
    if let Ok(rd) = fs::read_dir(&dir) {
        let mut paths: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
        paths.sort();
        for p in paths {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            if name.len() < 5 || !name[..4].chars().all(|c| c.is_ascii_digit()) || !name.ends_with(".md") {
                continue;
            }
            let text = fs::read_to_string(&p).unwrap_or_default();
            let title = text.lines().find(|l| l.starts_with("# ")).map(|l| l[2..].trim().to_string()).unwrap_or(name.clone());
            let status = text
                .lines()
                .find(|l| l.to_ascii_lowercase().starts_with("status:") || l.to_ascii_lowercase().starts_with("**status:**") || l.to_ascii_lowercase().starts_with("- status:"))
                .map(|l| {
                    let s = l.split(':').nth(1).unwrap_or("").trim().trim_end_matches("**").trim();
                    s.split(|c: char| c == ',' || c == '(').next().unwrap_or("").trim().to_string()
                })
                .unwrap_or_else(|| "Unknown".to_string());
            out.push((name[..4].to_string(), title, status));
        }
    }
    out
}

/// Go function inventory: package -> set of "file:name" keys.
fn read_inventory(root: &Path) -> BTreeMap<String, Vec<String>> {
    let p = root.join("data/go-functions.tsv");
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    if let Ok(text) = fs::read_to_string(&p) {
        for line in text.lines().skip(1) {
            let cols: Vec<&str> = line.split('\t').collect();
            if cols.len() < 4 {
                continue;
            }
            out.entry(cols[1].to_string()).or_default().push(format!("{}:{}", cols[0], cols[3]));
        }
    }
    out
}

/// Scan crates/**/*.rs for `port: <file>:<name>` markers.
fn scan_markers(root: &Path) -> Vec<String> {
    let mut markers = Vec::new();
    fn walk(dir: &Path, out: &mut Vec<String>) {
        if let Ok(rd) = fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    if p.file_name().map(|n| n == "target").unwrap_or(false) {
                        continue;
                    }
                    walk(&p, out);
                } else if p.extension().map(|x| x == "rs").unwrap_or(false) {
                    if let Ok(text) = fs::read_to_string(&p) {
                        for line in text.lines() {
                            let t = line.trim_start();
                            if let Some(rest) = t.strip_prefix("/// port:").or_else(|| t.strip_prefix("//! port:")).or_else(|| t.strip_prefix("// port:")) {
                                let m = rest.trim();
                                if m.contains(':') {
                                    out.push(m.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    walk(&root.join("crates"), &mut markers);
    markers.sort();
    markers.dedup();
    markers
}

// ---------------------------------------------------------------- checks

fn parse_number(s: &str) -> Option<f64> {
    s.parse::<f64>().ok()
}

/// Evaluate "<metric> <op> <value>". Returns None when the metric is unknown.
fn eval_check(metrics: &Metrics, check: &str) -> Option<bool> {
    let parts: Vec<&str> = check.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }
    let value = parts[parts.len() - 1];
    let op = parts[parts.len() - 2];
    let key = parts[..parts.len() - 2].join(" ");
    let m = metrics.get(&key)?;
    match m {
        Metric::Num(n) => {
            let v = if let Some(v) = parse_number(value) { v } else if status_rank(value) >= 0.0 { status_rank(value) } else { return None };
            Some(match op {
                "==" => (*n - v).abs() < 1e-9,
                "!=" => (*n - v).abs() >= 1e-9,
                ">=" => *n >= v,
                "<=" => *n <= v,
                ">" => *n > v,
                "<" => *n < v,
                _ => return None,
            })
        }
        Metric::Str(s) => {
            // Status strings compare by rank when both sides are statuses; otherwise as strings.
            let (a, b) = (status_rank(s), status_rank(value));
            if a >= 0.0 && b >= 0.0 {
                return Some(match op {
                    "==" => a == b,
                    "!=" => a != b,
                    ">=" => a >= b,
                    "<=" => a <= b,
                    ">" => a > b,
                    "<" => a < b,
                    _ => return None,
                });
            }
            Some(match op {
                "==" => s == value,
                "!=" => s != value,
                _ => return None,
            })
        }
    }
}

// ---------------------------------------------------------------- build report

fn build_report(root: &Path) -> Report {
    let ledger = read_ledger(root);
    let mut metrics: Metrics = BTreeMap::new();

    let in_scope: Vec<LedgerFile> = ledger.file.iter().filter(|f| f.kind != "out-of-scope" && f.status != "out-of-scope").cloned().collect();
    let counted: Vec<&LedgerFile> = in_scope.iter().filter(|f| f.kind == "source" || f.kind == "generated").collect();
    let total = counted.len() as f64;
    let ported = counted.iter().filter(|f| status_rank(&f.status) >= 2.0).count() as f64;
    let verified = counted.iter().filter(|f| f.status == "verified").count() as f64;
    let in_progress = counted.iter().filter(|f| f.status == "in-progress").count() as f64;
    let loc_total: f64 = counted.iter().map(|f| f.loc as f64).fold(0.0, |a, x| a + x);
    let loc_verified: f64 = counted.iter().filter(|f| f.status == "verified").map(|f| f.loc as f64).fold(0.0, |a, x| a + x);
    let loc_ported: f64 = counted.iter().filter(|f| status_rank(&f.status) >= 2.0).map(|f| f.loc as f64).fold(0.0, |a, x| a + x);
    let stale = counted.iter().filter(|f| !f.pin.is_empty() && !ledger.pin.is_empty() && f.pin != ledger.pin && status_rank(&f.status) >= 2.0).count() as f64;
    metrics.insert("ledger.files_total".into(), Metric::Num(total));
    metrics.insert("ledger.files_ported".into(), Metric::Num(ported));
    metrics.insert("ledger.files_verified".into(), Metric::Num(verified));
    metrics.insert("ledger.files_in_progress".into(), Metric::Num(in_progress));
    metrics.insert("ledger.files_stale".into(), Metric::Num(stale));
    metrics.insert("ledger.loc_total".into(), Metric::Num(loc_total));
    metrics.insert("ledger.loc_ported_ratio".into(), Metric::Num(ratio(loc_ported, loc_total)));
    metrics.insert("ledger.loc_verified_ratio".into(), Metric::Num(ratio(loc_verified, loc_total)));

    let mut by_phase: BTreeMap<i64, (f64, f64, f64)> = BTreeMap::new();
    let mut by_crate: BTreeMap<String, (f64, f64, f64)> = BTreeMap::new();
    for f in &counted {
        let e = by_phase.entry(f.phase).or_insert((0.0, 0.0, 0.0));
        e.0 += f.loc as f64;
        if f.status == "verified" {
            e.1 += f.loc as f64;
        }
        e.2 += 1.0;
        let c = by_crate.entry(f.krate.clone()).or_insert((0.0, 0.0, 0.0));
        c.0 += f.loc as f64;
        if f.status == "verified" {
            c.1 += f.loc as f64;
        }
        c.2 += 1.0;
    }
    for (ph, (t, v, n)) in &by_phase {
        metrics.insert(format!("ledger.phase[{ph}].loc_total"), Metric::Num(*t));
        metrics.insert(format!("ledger.phase[{ph}].files"), Metric::Num(*n));
        metrics.insert(format!("ledger.phase[{ph}].loc_verified_ratio"), Metric::Num(ratio(*v, *t)));
    }
    for (c, (t, v, n)) in &by_crate {
        metrics.insert(format!("ledger.crate[{c}].loc_total"), Metric::Num(*t));
        metrics.insert(format!("ledger.crate[{c}].files"), Metric::Num(*n));
        metrics.insert(format!("ledger.crate[{c}].loc_verified_ratio"), Metric::Num(ratio(*v, *t)));
    }
    for f in &ledger.file {
        metrics.insert(format!("file[{}].status", f.go), Metric::Str(f.status.clone()));
    }

    // Function-level traceability.
    let inventory = read_inventory(root);
    let markers = scan_markers(root);
    let source_files: std::collections::HashSet<&str> = counted.iter().filter(|f| f.kind == "source").map(|f| f.go.as_str()).collect();
    let mut all_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut fn_total = 0.0;
    let mut per_pkg_total: BTreeMap<String, f64> = BTreeMap::new();
    for (pkg, keys) in &inventory {
        for k in keys {
            let file = k.split(':').next().unwrap_or("");
            if source_files.contains(file) {
                all_keys.insert(k.clone());
                fn_total += 1.0;
                *per_pkg_total.entry(pkg.clone()).or_insert(0.0) += 1.0;
            }
        }
    }
    let mut fn_ported = 0.0;
    let mut per_pkg_ported: BTreeMap<String, f64> = BTreeMap::new();
    let mut ported_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut unknown = Vec::new();
    for m in &markers {
        if all_keys.contains(m) {
            if ported_keys.insert(m.clone()) {
                fn_ported += 1.0;
                let file = m.split(':').next().unwrap_or("");
                let pkg = file.trim_start_matches("tsc/").rsplit_once('/').map(|(d, _)| d.to_string()).unwrap_or_default();
                *per_pkg_ported.entry(pkg).or_insert(0.0) += 1.0;
            }
        } else {
            unknown.push(m.clone());
        }
    }
    metrics.insert("functions.total".into(), Metric::Num(fn_total));
    metrics.insert("functions.ported".into(), Metric::Num(fn_ported));
    metrics.insert("functions.ratio".into(), Metric::Num(ratio(fn_ported, fn_total)));
    let mut unported_by_package: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (pkg, keys) in &inventory {
        let t = per_pkg_total.get(pkg).copied().unwrap_or(0.0);
        if t == 0.0 {
            continue;
        }
        let p = per_pkg_ported.get(pkg).copied().unwrap_or(0.0);
        metrics.insert(format!("functions.package[{pkg}].total"), Metric::Num(t));
        metrics.insert(format!("functions.package[{pkg}].ported"), Metric::Num(p));
        metrics.insert(format!("functions.package[{pkg}].ratio"), Metric::Num(ratio(p, t)));
        let missing: Vec<String> = keys.iter().filter(|k| all_keys.contains(*k) && !ported_keys.contains(*k)).cloned().collect();
        if !missing.is_empty() {
            unported_by_package.insert(pkg.clone(), missing);
        }
    }

    // Experiments.
    let mut experiments = Vec::new();
    for (id, e) in read_experiments(root) {
        let pass = e.measured.map(|m| match e.op.as_str() {
            ">=" => m >= e.threshold,
            "<=" => m <= e.threshold,
            "==" => (m - e.threshold).abs() < 1e-9,
            ">" => m > e.threshold,
            "<" => m < e.threshold,
            _ => false,
        });
        metrics.insert(format!("exp.{id}.threshold"), Metric::Num(e.threshold));
        if let Some(m) = e.measured {
            metrics.insert(format!("exp.{id}.measured"), Metric::Num(m));
        }
        metrics.insert(format!("exp.{id}.pass"), Metric::Num(if pass == Some(true) { 1.0 } else { 0.0 }));
        experiments.push((id, e, pass));
    }
    let exp_pass = experiments.iter().filter(|(_, _, p)| *p == Some(true)).count() as f64;
    metrics.insert("exp.passed".into(), Metric::Num(exp_pass));
    metrics.insert("exp.total".into(), Metric::Num(experiments.len() as f64));

    // ADRs.
    let adrs = read_adrs(root);
    for (n, _, st) in &adrs {
        metrics.insert(format!("adr.{n}.status"), Metric::Str(st.clone()));
    }
    metrics.insert("adr.total".into(), Metric::Num(adrs.len() as f64));
    metrics.insert("adr.accepted".into(), Metric::Num(adrs.iter().filter(|(_, _, s)| s == "Accepted" || s == "Amended").count() as f64));

    // Sprints (evaluated after all other metrics exist).
    let mut sprints = Vec::new();
    for s in read_sprints(root) {
        let exit: Vec<(String, Option<bool>)> = s.exit.iter().map(|c| (c.clone(), eval_check(&metrics, c))).collect();
        let done = !exit.is_empty() && exit.iter().all(|(_, r)| *r == Some(true));
        let items: Vec<(String, String, Option<bool>)> = s
            .item
            .iter()
            .map(|it| {
                let r = if it.done_when.is_empty() {
                    None
                } else {
                    let rs: Vec<Option<bool>> = it.done_when.iter().map(|c| eval_check(&metrics, c)).collect();
                    Some(rs.iter().all(|r| *r == Some(true)))
                };
                let label = if it.r#ref.is_empty() { it.title.clone() } else { format!("{} ({} {})", it.title, it.kind, it.r#ref) };
                (it.id.clone(), label, r)
            })
            .collect();
        metrics.insert(format!("sprint.{}.done", s.id), Metric::Num(if done { 1.0 } else { 0.0 }));
        sprints.push(SprintResult { id: s.id, title: s.title, goal: s.goal, done, exit, items });
    }

    Report { metrics, ledger_pin: ledger.pin, files_in_scope: in_scope, unported_by_package, unknown_markers: unknown, experiments, sprints, adrs }
}

// ---------------------------------------------------------------- outputs

fn num(m: &Metrics, k: &str) -> f64 {
    match m.get(k) {
        Some(Metric::Num(n)) => *n,
        _ => 0.0,
    }
}

fn pct(x: f64) -> String {
    format!("{:.1}%", x * 100.0)
}

fn today() -> String {
    // Civil date from the Unix epoch (Howard Hinnant's algorithm), UTC.
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let z = secs.div_euclid(86_400) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

fn write_status_md(root: &Path, r: &Report) {
    let m = &r.metrics;
    let mut s = String::new();
    s.push_str("# Status\n\n");
    s.push_str(&format!("Generated by `cargo xtask status` on {} against upstream pin `{}`. Do not edit; edit the ledger, the sprint files, the experiments file or the ADRs.\n\n", today(), r.ledger_pin));
    s.push_str("## Summary\n\n| | |\n|---|---|\n");
    s.push_str(&format!("| Files in scope (source and generated) | {} |\n", num(m, "ledger.files_total")));
    s.push_str(&format!("| Files ported / verified | {} / {} |\n", num(m, "ledger.files_ported"), num(m, "ledger.files_verified")));
    s.push_str(&format!("| Lines verified | {} of {} ({}) |\n", num(m, "ledger.loc_total") * num(m, "ledger.loc_verified_ratio"), num(m, "ledger.loc_total"), pct(num(m, "ledger.loc_verified_ratio"))));
    s.push_str(&format!("| Functions with a Rust counterpart | {} of {} ({}) |\n", num(m, "functions.ported"), num(m, "functions.total"), pct(num(m, "functions.ratio"))));
    s.push_str(&format!("| Experiments passed | {} of {} |\n", num(m, "exp.passed"), num(m, "exp.total")));
    s.push_str(&format!("| ADRs accepted | {} of {} |\n", num(m, "adr.accepted"), num(m, "adr.total")));
    s.push_str(&format!("| Entries stale against the pin | {} |\n\n", num(m, "ledger.files_stale")));

    s.push_str("## By phase\n\n| Phase | Files | Lines | Verified |\n|---|---:|---:|---:|\n");
    let mut phases: Vec<i64> = r.files_in_scope.iter().filter(|f| f.kind == "source" || f.kind == "generated").map(|f| f.phase).collect();
    phases.sort();
    phases.dedup();
    for ph in phases {
        s.push_str(&format!("| {} | {} | {} | {} |\n", ph, num(m, &format!("ledger.phase[{ph}].files")), num(m, &format!("ledger.phase[{ph}].loc_total")), pct(num(m, &format!("ledger.phase[{ph}].loc_verified_ratio")))));
    }

    s.push_str("\n## By crate\n\n| Crate | Files | Lines | Verified |\n|---|---:|---:|---:|\n");
    let mut crates: Vec<String> = r.files_in_scope.iter().filter(|f| f.kind == "source" || f.kind == "generated").map(|f| f.krate.clone()).collect();
    crates.sort();
    crates.dedup();
    for c in crates {
        s.push_str(&format!("| `{}` | {} | {} | {} |\n", c, num(m, &format!("ledger.crate[{c}].files")), num(m, &format!("ledger.crate[{c}].loc_total")), pct(num(m, &format!("ledger.crate[{c}].loc_verified_ratio")))));
    }

    s.push_str("\n## Function coverage by package\n\n| Package | Functions | Ported |\n|---|---:|---:|\n");
    let mut pkgs: Vec<String> = m.keys().filter_map(|k| k.strip_prefix("functions.package[").and_then(|k| k.split(']').next()).map(|s| s.to_string())).collect();
    pkgs.sort();
    pkgs.dedup();
    for p in pkgs {
        s.push_str(&format!("| `{}` | {} | {} |\n", p, num(m, &format!("functions.package[{p}].total")), pct(num(m, &format!("functions.package[{p}].ratio")))));
    }
    if !r.unknown_markers.is_empty() {
        s.push_str("\n**Unknown markers** (name no upstream function; stale after a pin bump?):\n\n");
        for u in &r.unknown_markers {
            s.push_str(&format!("- `{u}`\n"));
        }
    }

    s.push_str("\n## Experiments\n\n| | Title | Threshold | Measured | Pass | Nature |\n|---|---|---|---|---|---|\n");
    for (id, e, pass) in &r.experiments {
        let measured = e.measured.map(|v| format!("{v} ({})", e.measured_at)).unwrap_or_else(|| "not yet".into());
        let p = match pass {
            Some(true) => "yes",
            Some(false) => "no",
            None => "pending",
        };
        s.push_str(&format!("| {} | {} | {} {} {} | {} | {} | {} |\n", id, e.title, e.op, e.threshold, e.unit, measured, p, e.nature));
    }

    s.push_str("\n## Sprints\n\n");
    for sp in &r.sprints {
        s.push_str(&format!("### {} {} {}\n\n{}\n\n", sp.id, sp.title, if sp.done { "(done)" } else { "(open)" }, sp.goal));
        s.push_str("Exit checks:\n\n");
        for (c, res) in &sp.exit {
            s.push_str(&format!("- [{}] `{}`{}\n", if *res == Some(true) { "x" } else { " " }, c, if res.is_none() { " (unknown metric)" } else { "" }));
        }
        s.push_str("\nItems:\n\n");
        for (id, label, res) in &sp.items {
            let mark = match res {
                Some(true) => "x",
                Some(false) => " ",
                None => "-",
            };
            s.push_str(&format!("- [{}] {} {}\n", mark, id, label));
        }
        s.push('\n');
    }

    s.push_str("## Decisions\n\n| ADR | Title | Status |\n|---|---|---|\n");
    for (n, t, st) in &r.adrs {
        s.push_str(&format!("| {} | {} | {} |\n", n, t, st));
    }
    fs::write(root.join("STATUS.md"), s).unwrap_or_else(|e| die(&format!("STATUS.md: {e}")));
}

fn write_status_json(root: &Path, r: &Report) {
    let mut obj = serde_json::Map::new();
    obj.insert("generated".into(), serde_json::Value::String(today()));
    obj.insert("pin".into(), serde_json::Value::String(r.ledger_pin.clone()));
    let mut metrics = serde_json::Map::new();
    for (k, v) in &r.metrics {
        if k.starts_with("file[") {
            continue;
        }
        metrics.insert(k.clone(), serde_json::to_value(v).unwrap());
    }
    obj.insert("metrics".into(), serde_json::Value::Object(metrics));
    let unported: serde_json::Map<String, serde_json::Value> = r.unported_by_package.iter().map(|(k, v)| (k.clone(), serde_json::json!(v.len()))).collect();
    obj.insert("unported_functions_by_package".into(), serde_json::Value::Object(unported));
    fs::create_dir_all(root.join("status")).ok();
    fs::write(root.join("status/status.json"), serde_json::to_string_pretty(&serde_json::Value::Object(obj)).unwrap()).unwrap_or_else(|e| die(&format!("status.json: {e}")));
}

fn record_history(root: &Path, r: &Report) {
    let m = &r.metrics;
    let line = serde_json::json!({
        "date": today(),
        "pin": r.ledger_pin,
        "loc_verified_ratio": num(m, "ledger.loc_verified_ratio"),
        "loc_ported_ratio": num(m, "ledger.loc_ported_ratio"),
        "functions_ratio": num(m, "functions.ratio"),
        "files_verified": num(m, "ledger.files_verified"),
        "exp_passed": num(m, "exp.passed"),
    });
    let p = root.join("status/history.jsonl");
    let mut text = fs::read_to_string(&p).unwrap_or_default();
    text.push_str(&line.to_string());
    text.push('\n');
    fs::write(&p, text).unwrap_or_else(|e| die(&format!("history.jsonl: {e}")));
}

fn write_dashboard(root: &Path, r: &Report) {
    let m = &r.metrics;
    let history = fs::read_to_string(root.join("status/history.jsonl")).unwrap_or_default();
    let points: Vec<(String, f64, f64)> = history
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .map(|v| (v["date"].as_str().unwrap_or("").to_string(), v["loc_verified_ratio"].as_f64().unwrap_or(0.0), v["functions_ratio"].as_f64().unwrap_or(0.0)))
        .collect();

    let chart = {
        let w = 720.0;
        let h = 220.0;
        let n = points.len().max(2) as f64;
        let mut poly_loc = String::new();
        let mut poly_fn = String::new();
        for (i, (_, loc, f)) in points.iter().enumerate() {
            let x = 40.0 + (w - 60.0) * (i as f64) / (n - 1.0);
            poly_loc.push_str(&format!("{:.1},{:.1} ", x, 10.0 + (h - 40.0) * (1.0 - loc)));
            poly_fn.push_str(&format!("{:.1},{:.1} ", x, 10.0 + (h - 40.0) * (1.0 - f)));
        }
        let first = points.first().map(|p| p.0.clone()).unwrap_or_default();
        let last = points.last().map(|p| p.0.clone()).unwrap_or_default();
        format!(
            r#"<svg viewBox="0 0 {w} {h}" width="100%" role="img" aria-label="Verified lines and ported functions over time">
  <line x1="40" y1="10" x2="40" y2="{y0}" stroke="var(--line)"/><line x1="40" y1="{y0}" x2="{x1}" y2="{y0}" stroke="var(--line)"/>
  <text x="4" y="14" class="ax">100%</text><text x="4" y="{ymid}" class="ax">50%</text><text x="12" y="{y0}" class="ax">0%</text>
  <text x="40" y="{yl}" class="ax">{first}</text><text x="{x1}" y="{yl}" class="ax" text-anchor="end">{last}</text>
  <polyline fill="none" stroke="var(--accent)" stroke-width="2" points="{poly_loc}"/>
  <polyline fill="none" stroke="var(--rust)" stroke-width="2" stroke-dasharray="4 3" points="{poly_fn}"/>
</svg>
<div class="legend"><span class="acc">verified lines</span><span class="rust">ported functions</span></div>"#,
            w = w,
            h = h,
            y0 = h - 30.0,
            ymid = 10.0 + (h - 40.0) / 2.0,
            yl = h - 12.0,
            x1 = w - 20.0
        )
    };

    let mut phase_rows = String::new();
    let mut phases: Vec<i64> = r.files_in_scope.iter().filter(|f| f.kind == "source" || f.kind == "generated").map(|f| f.phase).collect();
    phases.sort();
    phases.dedup();
    for ph in phases {
        let v = num(m, &format!("ledger.phase[{ph}].loc_verified_ratio"));
        phase_rows.push_str(&format!(
            "<tr><td>Phase {ph}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td><div class=\"bar\"><div style=\"width:{:.1}%\"></div></div></td><td class=\"num\">{}</td></tr>\n",
            num(m, &format!("ledger.phase[{ph}].files")),
            num(m, &format!("ledger.phase[{ph}].loc_total")),
            v * 100.0,
            pct(v)
        ));
    }
    let mut exp_rows = String::new();
    for (id, e, pass) in &r.experiments {
        let cls = match pass {
            Some(true) => "ok",
            Some(false) => "crit",
            None => "warn",
        };
        let label = match pass {
            Some(true) => "pass",
            Some(false) => "fail",
            None => "pending",
        };
        exp_rows.push_str(&format!(
            "<tr><td>{id}</td><td>{}</td><td>{} {} {}</td><td>{}</td><td><span class=\"chip {cls}\">{label}</span></td></tr>\n",
            e.title,
            e.op,
            e.threshold,
            e.unit,
            e.measured.map(|v| v.to_string()).unwrap_or_else(|| "not yet".into())
        ));
    }
    let mut sprint_rows = String::new();
    for sp in &r.sprints {
        let passed = sp.exit.iter().filter(|(_, r)| *r == Some(true)).count();
        sprint_rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td class=\"num\">{} / {}</td><td><span class=\"chip {}\">{}</span></td></tr>\n",
            sp.id,
            sp.title,
            passed,
            sp.exit.len(),
            if sp.done { "ok" } else { "warn" },
            if sp.done { "done" } else { "open" }
        ));
    }

    let html = format!(
        r#"<title>Corsa in Rust Status</title>
<style>
  :root {{ --bg:#f6f7f9; --surface:#fff; --ink:#1a2029; --ink-2:#4b5665; --line:#d7dde6; --accent:#245fa6; --rust:#b7410e; --ok:#2f6f3e; --ok-soft:#e1f2e5; --warn:#8a5a00; --warn-soft:#fbefd2; --crit:#b42318; --crit-soft:#fde8e6; color-scheme: light dark; }}
  @media (prefers-color-scheme: dark) {{ :root:not([data-theme="light"]) {{ --bg:#101418; --surface:#161b21; --ink:#e4e8ee; --ink-2:#a6b0be; --line:#2a323c; --accent:#7ab4f5; --rust:#e8875a; --ok:#7cc58f; --ok-soft:#1a3322; --warn:#e2b93b; --warn-soft:#3a2f12; --crit:#f08a80; --crit-soft:#3d1d1a; }} }}
  :root[data-theme="dark"] {{ --bg:#101418; --surface:#161b21; --ink:#e4e8ee; --ink-2:#a6b0be; --line:#2a323c; --accent:#7ab4f5; --rust:#e8875a; --ok:#7cc58f; --ok-soft:#1a3322; --warn:#e2b93b; --warn-soft:#3a2f12; --crit:#f08a80; --crit-soft:#3d1d1a; }}
  body {{ margin:0; background:var(--bg); color:var(--ink); font:400 15px/1.5 "IBM Plex Sans", system-ui, sans-serif; }}
  .page {{ max-width:1000px; margin:0 auto; padding:32px 24px 60px; }}
  h1 {{ font:600 30px/1.2 "IBM Plex Serif", Georgia, serif; margin:0 0 6px; }}
  h2 {{ font:600 20px/1.3 "IBM Plex Serif", Georgia, serif; margin:28px 0 10px; }}
  .eyebrow {{ font-size:12px; letter-spacing:.08em; text-transform:uppercase; color:var(--ink-2); }}
  .tiles {{ display:grid; grid-template-columns:repeat(auto-fit,minmax(180px,1fr)); gap:10px; margin:18px 0; }}
  .tile {{ background:var(--surface); border:1px solid var(--line); border-radius:4px; padding:12px 14px; }}
  .tile b {{ display:block; font:500 24px/1.1 "JetBrains Mono", ui-monospace, monospace; margin-bottom:4px; }}
  .tile span {{ color:var(--ink-2); font-size:13px; }}
  table {{ border-collapse:collapse; width:100%; font-size:14px; }}
  th, td {{ text-align:left; padding:7px 10px 7px 0; border-bottom:1px solid var(--line); vertical-align:top; }}
  th {{ font-size:12px; letter-spacing:.05em; text-transform:uppercase; color:var(--ink-2); }}
  td.num {{ text-align:right; font-variant-numeric:tabular-nums; white-space:nowrap; padding-right:16px; }}
  .bar {{ background:var(--line); height:8px; border-radius:2px; min-width:160px; }}
  .bar div {{ background:var(--accent); height:8px; border-radius:2px; }}
  .chip {{ display:inline-block; font-size:11px; letter-spacing:.04em; text-transform:uppercase; padding:3px 6px; border-radius:3px; }}
  .chip.ok {{ background:var(--ok-soft); color:var(--ok); }} .chip.warn {{ background:var(--warn-soft); color:var(--warn); }} .chip.crit {{ background:var(--crit-soft); color:var(--crit); }}
  .ax {{ font:11px ui-monospace, monospace; fill:var(--ink-2); }}
  .legend {{ font-size:13px; color:var(--ink-2); display:flex; gap:16px; }}
  .legend span::before {{ content:""; display:inline-block; width:18px; height:3px; margin-right:6px; vertical-align:middle; background:var(--accent); }}
  .legend .rust::before {{ background:var(--rust); }}
  .chart {{ background:var(--surface); border:1px solid var(--line); border-radius:4px; padding:12px; }}
</style>
<div class="page">
  <div class="eyebrow">Corsa in Rust &middot; generated {date} &middot; upstream pin {pin}</div>
  <h1>Status</h1>
  <div class="tiles">
    <div class="tile"><b>{loc_pct}</b><span>lines verified, of {loc_total}</span></div>
    <div class="tile"><b>{files_verified} / {files_total}</b><span>files verified</span></div>
    <div class="tile"><b>{fn_pct}</b><span>functions with a Rust counterpart, of {fn_total}</span></div>
    <div class="tile"><b>{exp_passed} / {exp_total}</b><span>experiments passed</span></div>
    <div class="tile"><b>{adr_acc} / {adr_total}</b><span>decisions accepted</span></div>
  </div>
  <h2>Over time</h2>
  <div class="chart">{chart}</div>
  <h2>By phase</h2>
  <table><thead><tr><th>Phase</th><th class="num">Files</th><th class="num">Lines</th><th>Verified</th><th class="num"></th></tr></thead><tbody>
{phase_rows}</tbody></table>
  <h2>Experiments</h2>
  <table><thead><tr><th></th><th>Title</th><th>Threshold</th><th>Measured</th><th>Result</th></tr></thead><tbody>
{exp_rows}</tbody></table>
  <h2>Sprints</h2>
  <table><thead><tr><th>Sprint</th><th>Title</th><th class="num">Exit checks</th><th>State</th></tr></thead><tbody>
{sprint_rows}</tbody></table>
</div>
"#,
        date = today(),
        pin = r.ledger_pin,
        loc_pct = pct(num(m, "ledger.loc_verified_ratio")),
        loc_total = num(m, "ledger.loc_total"),
        files_verified = num(m, "ledger.files_verified"),
        files_total = num(m, "ledger.files_total"),
        fn_pct = pct(num(m, "functions.ratio")),
        fn_total = num(m, "functions.total"),
        exp_passed = num(m, "exp.passed"),
        exp_total = num(m, "exp.total"),
        adr_acc = num(m, "adr.accepted"),
        adr_total = num(m, "adr.total"),
    );
    fs::create_dir_all(root.join("docs")).ok();
    fs::write(root.join("docs/status.html"), html).unwrap_or_else(|e| die(&format!("status.html: {e}")));
}

// ---------------------------------------------------------------- main

fn die(msg: &str) -> ! {
    eprintln!("xtask: {msg}");
    std::process::exit(2)
}

fn repo_root() -> PathBuf {
    let manifest = env!("CARGO_MANIFEST_DIR");
    Path::new(manifest).parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = repo_root();
    match args.first().map(|s| s.as_str()) {
        Some("status") => {
            let r = build_report(&root);
            if !r.unknown_markers.is_empty() {
                eprintln!("xtask: {} unknown port markers (listed in STATUS.md)", r.unknown_markers.len());
            }
            write_status_md(&root, &r);
            write_status_json(&root, &r);
            if args.iter().any(|a| a == "--record") {
                record_history(&root, &r);
            }
            write_dashboard(&root, &r);
            let m = &r.metrics;
            println!(
                "status: {} files in scope, {} verified ({} of lines), {} of {} functions ported, {} of {} experiments passed, {} sprints",
                num(m, "ledger.files_total"),
                num(m, "ledger.files_verified"),
                pct(num(m, "ledger.loc_verified_ratio")),
                num(m, "functions.ported"),
                num(m, "functions.total"),
                num(m, "exp.passed"),
                num(m, "exp.total"),
                r.sprints.len()
            );
            if r.unknown_markers.is_empty() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Some("check") => {
            let id = args.get(1).cloned().unwrap_or_else(|| die("usage: xtask check <sprint id>"));
            let r = build_report(&root);
            match r.sprints.iter().find(|s| s.id == id) {
                Some(s) => {
                    for (c, res) in &s.exit {
                        println!("{} {}", match res { Some(true) => "pass ", Some(false) => "FAIL ", None => "??   " }, c);
                    }
                    if s.done {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(1)
                    }
                }
                None => die(&format!("no sprint {id}")),
            }
        }
        _ => {
            eprintln!("usage: cargo xtask status [--record] | cargo xtask check <sprint id>");
            ExitCode::from(2)
        }
    }
}
