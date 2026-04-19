// Baseline stress test for the four consensus protocols shipped in
// libchatter-rs: Apollo, Artemis, Sync HotStuff, and Opt Sync.
//
// For each protocol, the harness:
//   1. Shells out to `genconfig` to produce a fresh Node/Client config set
//   2. Writes matching ip / cli_ip files for localhost loopback
//   3. Spawns N node binaries as child processes
//   4. Spawns one client binary, which drives load until `-m` transactions commit
//   5. Parses `DP[Throughput]` / `DP[Latency]` lines printed by
//      `consensus::statistics` on the client's stderr (simple_logger at INFO)
//   6. Kills nodes, reports result, moves on
//
// The output format mirrors libnet-rs's stress-test so the two baselines
// can sit side-by-side in README / CV material. The canonical run is
// captured in `baseline_results.txt` at the repo root.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::{sleep, timeout};

type BoxErr = Box<dyn Error + Send + Sync>;

#[derive(Clone, Copy, Debug)]
enum Protocol {
    Apollo,
    Artemis,
    Synchs,
    Optsync,
}

impl Protocol {
    fn short(&self) -> &'static str {
        match self {
            Protocol::Apollo => "apollo",
            Protocol::Artemis => "artemis",
            Protocol::Synchs => "synchs",
            Protocol::Optsync => "optsync",
        }
    }
    fn label(&self) -> &'static str {
        match self {
            Protocol::Apollo => "Apollo",
            Protocol::Artemis => "Artemis",
            Protocol::Synchs => "Sync HotStuff",
            Protocol::Optsync => "Opt Sync",
        }
    }
    // Seconds the nodes wait at startup before entering the protocol loop.
    // Apollo and Artemis gate protocol entry on the `--sleep` arg; Sync HotStuff
    // and Opt Sync gate on `config::SLEEP_TIME`, which the binaries compute from
    // `--sleep` when present. A larger N needs more slack.
    fn bootstrap_secs(num_nodes: usize) -> u64 {
        5 + (num_nodes as u64).max(3)
    }

    // Apollo and Artemis have two client-notification paths: the default
    // (commit-driven) requires round > f before the leader ever tells the
    // client about a block, which deadlocks with a single initial-tx burst
    // because no node has enough txs left in its pool to propose round f+1.
    // The `-s` flag ("special_client") enables the fast path where the leader
    // multicasts proposals to clients immediately. This matches the canonical
    // `scripts/apollo-multi-node-test.sh` behaviour. Sync HotStuff and Opt Sync
    // always multicast blocks to clients, so they don't have this flag.
    fn wants_special_client(&self) -> bool {
        matches!(self, Protocol::Apollo | Protocol::Artemis)
    }
}

#[derive(Clone, Debug)]
struct BenchConfig {
    protocol: Protocol,
    num_nodes: usize,
    num_faults: usize,
    block_size: usize,
    payload: usize,
    total_txs: u64,
    window: usize,
}

struct BenchResult {
    throughput: f64, // tx / sec
    latency_ms: f64, // avg ms per tx
    wall_elapsed: Duration,
}

struct Harness {
    repo_root: PathBuf,
    runs_dir: PathBuf,
    run_idx: u16,
}

impl Harness {
    fn new(repo_root: PathBuf) -> Result<Self, BoxErr> {
        let runs_dir = repo_root.join("stress-test/runs");
        fs::create_dir_all(&runs_dir)?;
        Ok(Self {
            repo_root,
            runs_dir,
            run_idx: 0,
        })
    }

    // Allocate a non-overlapping port block per run: 200 ports each, starting
    // high enough that we don't collide with typical dev services.
    fn alloc_ports(&mut self) -> (u16, u16) {
        let base = 21000 + self.run_idx * 200;
        let cli_base = base + 100;
        self.run_idx += 1;
        (base, cli_base)
    }

    async fn run(&mut self, cfg: &BenchConfig) -> Result<BenchResult, BoxErr> {
        let (base_port, cli_base_port) = self.alloc_ports();
        let run_dir = self.runs_dir.join(format!(
            "{}-n{}-b{}-p{}-{}",
            cfg.protocol.short(),
            cfg.num_nodes,
            cfg.block_size,
            cfg.payload,
            base_port
        ));
        if run_dir.exists() {
            fs::remove_dir_all(&run_dir).ok();
        }
        fs::create_dir_all(&run_dir)?;

        genconfig(&self.repo_root, &run_dir, cfg, base_port, cli_base_port).await?;
        write_ip_files(&run_dir, cfg.num_nodes, base_port, cli_base_port)?;

        let bootstrap = Protocol::bootstrap_secs(cfg.num_nodes);
        let mut nodes: Vec<Child> = Vec::with_capacity(cfg.num_nodes);
        for i in 0..cfg.num_nodes {
            nodes.push(spawn_node(&self.repo_root, &run_dir, cfg, i, bootstrap).await?);
        }

        sleep(Duration::from_secs(bootstrap)).await;

        let started = Instant::now();
        let client_out = spawn_client_and_parse(&self.repo_root, &run_dir, cfg).await;
        let wall_elapsed = started.elapsed();

        for mut n in nodes {
            let _ = n.kill().await;
        }
        // Give TCP listeners a moment to release their ports before the next run.
        sleep(Duration::from_millis(500)).await;

        let (throughput, latency_ms) = client_out?;
        Ok(BenchResult {
            throughput,
            latency_ms,
            wall_elapsed,
        })
    }
}

async fn genconfig(
    repo_root: &Path,
    run_dir: &Path,
    cfg: &BenchConfig,
    base_port: u16,
    cli_base_port: u16,
) -> Result<(), BoxErr> {
    let bin = repo_root.join("target/release/genconfig");
    let out = Command::new(&bin)
        .arg("-n")
        .arg(cfg.num_nodes.to_string())
        .arg("-f")
        .arg(cfg.num_faults.to_string())
        .arg("-d")
        .arg("50")
        .arg("--blocksize")
        .arg(cfg.block_size.to_string())
        .arg("--base_port")
        .arg(base_port.to_string())
        .arg("--client_base_port")
        .arg(cli_base_port.to_string())
        .arg("--payload")
        .arg(cfg.payload.to_string())
        .arg("--target")
        .arg(run_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("genconfig exited non-zero: {}", stderr).into());
    }
    Ok(())
}

fn write_ip_files(
    run_dir: &Path,
    n: usize,
    base_port: u16,
    cli_base_port: u16,
) -> Result<(), BoxErr> {
    let mut ip = String::new();
    let mut cli_ip = String::new();
    for i in 0..n {
        ip.push_str(&format!("127.0.0.1:{}\n", base_port as usize + i));
        cli_ip.push_str(&format!("127.0.0.1:{}\n", cli_base_port as usize + i));
    }
    fs::write(run_dir.join("ip_file"), ip)?;
    fs::write(run_dir.join("cli_ip_file"), cli_ip)?;
    Ok(())
}

async fn spawn_node(
    repo_root: &Path,
    run_dir: &Path,
    cfg: &BenchConfig,
    i: usize,
    bootstrap_secs: u64,
) -> Result<Child, BoxErr> {
    let bin = repo_root.join(format!("target/release/node-{}", cfg.protocol.short()));
    let config_file = run_dir.join(format!("nodes-{}.json", i));
    let ip_file = run_dir.join("ip_file");
    let log_path = run_dir.join(format!("node-{}.log", i));
    let stdout_fd = std::fs::File::create(&log_path)?;
    let stderr_fd = stdout_fd.try_clone()?;

    let mut cmd = Command::new(&bin);
    cmd.arg("-c")
        .arg(&config_file)
        .arg("-i")
        .arg(&ip_file)
        .arg("--sleep")
        .arg(bootstrap_secs.to_string())
        .arg("--delta")
        .arg("50");
    if cfg.protocol.wants_special_client() {
        cmd.arg("-s");
    }
    let child = cmd
        .stdout(Stdio::from(stdout_fd))
        .stderr(Stdio::from(stderr_fd))
        .spawn()?;
    Ok(child)
}

async fn spawn_client_and_parse(
    repo_root: &Path,
    run_dir: &Path,
    cfg: &BenchConfig,
) -> Result<(f64, f64), BoxErr> {
    let bin = repo_root.join(format!("target/release/client-{}", cfg.protocol.short()));
    let config_file = run_dir.join("client.json");
    let cli_ip_file = run_dir.join("cli_ip_file");

    let log_path = run_dir.join("client.log");
    let mut child = Command::new(&bin)
        .arg("-c")
        .arg(&config_file)
        .arg("-i")
        .arg(&cli_ip_file)
        .arg("-m")
        .arg(cfg.total_txs.to_string())
        .arg("-w")
        .arg(cfg.window.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stderr = child
        .stderr
        .take()
        .ok_or("client stderr not captured")?;
    let stdout = child
        .stdout
        .take()
        .ok_or("client stdout not captured")?;
    let mut stderr_lines = BufReader::new(stderr).lines();
    let mut stdout_lines = BufReader::new(stdout).lines();

    // Tee everything the client prints into a log file so post-mortems are possible.
    let mut log = std::fs::File::create(&log_path)?;

    let parse_window = Duration::from_secs(600);
    let mut throughput: Option<f64> = None;
    let mut latency: Option<f64> = None;

    let parse = async {
        use std::io::Write;
        loop {
            tokio::select! {
                line = stderr_lines.next_line() => {
                    match line {
                        Ok(Some(l)) => {
                            let _ = writeln!(log, "[stderr] {}", l);
                            if let Some(v) = extract_after(&l, "DP[Throughput]: ") {
                                throughput = Some(v);
                            }
                            if let Some(v) = extract_after(&l, "DP[Latency]: ") {
                                latency = Some(v);
                            }
                            if throughput.is_some() && latency.is_some() {
                                break;
                            }
                        }
                        Ok(None) => break,
                        Err(_) => break,
                    }
                }
                line = stdout_lines.next_line() => {
                    match line {
                        Ok(Some(l)) => {
                            let _ = writeln!(log, "[stdout] {}", l);
                            if let Some(v) = extract_after(&l, "DP[Throughput]: ") {
                                throughput = Some(v);
                            }
                            if let Some(v) = extract_after(&l, "DP[Latency]: ") {
                                latency = Some(v);
                            }
                            if throughput.is_some() && latency.is_some() {
                                break;
                            }
                        }
                        Ok(None) => break,
                        Err(_) => break,
                    }
                }
            }
        }
    };

    match timeout(parse_window, parse).await {
        Ok(()) => {}
        Err(_) => {
            let _ = child.kill().await;
            return Err("client timed out before reporting DP stats".into());
        }
    }

    // `consensus::statistics` prints, then the `start` future returns, then
    // the tokio runtime drops the dangling tx-producer task which calls
    // `std::process::exit(0)`. Kill explicitly in case any of that gets stuck.
    let _ = child.kill().await;

    match (throughput, latency) {
        (Some(t), Some(l)) => Ok((t, l)),
        (Some(_), None) => Err("saw DP[Throughput] but not DP[Latency]".into()),
        (None, Some(_)) => Err("saw DP[Latency] but not DP[Throughput]".into()),
        (None, None) => Err("client produced no DP lines (check run_dir/client.log)".into()),
    }
}

fn extract_after(line: &str, prefix: &str) -> Option<f64> {
    let idx = line.find(prefix)?;
    let rest = &line[idx + prefix.len()..];
    rest.trim().parse::<f64>().ok()
}

// ---- Reporting ----

const BOX_WIDTH: usize = 63;

fn box_line() -> String {
    "─".repeat(BOX_WIDTH)
}

fn print_header(cfg: &BenchConfig) {
    println!();
    println!("┌{}", box_line());
    println!(
        "│ {} (n={}, f={}, blk={}, payload={}B, window={}, txs={})",
        cfg.protocol.label(),
        cfg.num_nodes,
        cfg.num_faults,
        cfg.block_size,
        cfg.payload,
        cfg.window,
        cfg.total_txs
    );
    println!("├{}", box_line());
}

fn print_result(r: &BenchResult) {
    let mb_per_sec = 0.0; // block payload is opaque; leave blank
    let _ = mb_per_sec;
    println!("│ Throughput     : {:>12.2} tx/s", r.throughput);
    println!("│ Avg Latency    : {:>12.2} ms/tx", r.latency_ms);
    println!("│ Wall elapsed   : {:>12.2} s", r.wall_elapsed.as_secs_f64());
    println!("└{}", box_line());
}

fn print_failure(err: &(dyn Error + Send + Sync)) {
    println!("│ FAILED: {}", err);
    println!("└{}", box_line());
}

fn print_summary(results: &[(BenchConfig, Option<BenchResult>)]) {
    let w = 90;
    let line = "─".repeat(w);
    println!();
    println!("┌{}", line);
    println!(
        "│ {:<16} {:>4} {:>4} {:>5} {:>7} {:>16} {:>16}",
        "Protocol", "N", "f", "blk", "txs", "Throughput", "Latency"
    );
    println!("├{}", line);
    for (c, r) in results {
        match r {
            Some(r) => println!(
                "│ {:<16} {:>4} {:>4} {:>5} {:>7} {:>10.2} tx/s {:>11.2} ms",
                c.protocol.label(),
                c.num_nodes,
                c.num_faults,
                c.block_size,
                c.total_txs,
                r.throughput,
                r.latency_ms
            ),
            None => println!(
                "│ {:<16} {:>4} {:>4} {:>5} {:>7} {:>16} {:>16}",
                c.protocol.label(),
                c.num_nodes,
                c.num_faults,
                c.block_size,
                c.total_txs,
                "FAILED",
                "FAILED"
            ),
        }
    }
    println!("└{}", line);
}

// ---- Matrix ----

fn build_matrix() -> Vec<BenchConfig> {
    let mut v = Vec::new();
    for &protocol in &[
        Protocol::Apollo,
        Protocol::Artemis,
        Protocol::Synchs,
        Protocol::Optsync,
    ] {
        for &(n, f) in &[(3usize, 1usize), (7, 3)] {
            v.push(BenchConfig {
                protocol,
                num_nodes: n,
                num_faults: f,
                block_size: 400,
                payload: 0,
                total_txs: 50_000,
                window: 10_000,
            });
        }
    }
    v
}

#[tokio::main]
async fn main() -> Result<(), BoxErr> {
    let repo_root = std::env::current_dir()?;

    // Sanity-check binaries so the first failure is informative, not a cryptic ENOENT.
    for bin in &[
        "genconfig",
        "node-apollo",
        "client-apollo",
        "node-artemis",
        "client-artemis",
        "node-synchs",
        "client-synchs",
        "node-optsync",
        "client-optsync",
    ] {
        let p = repo_root.join(format!("target/release/{}", bin));
        if !p.exists() {
            return Err(format!(
                "missing target/release/{}. Run `cargo build --release` first.",
                bin
            )
            .into());
        }
    }

    let mut harness = Harness::new(repo_root)?;
    let matrix = build_matrix();

    println!("{:=^63}", " libchatter-rs baseline stress test ");
    println!(
        "Protocols: Apollo, Artemis, Sync HotStuff, Opt Sync.     Loopback (127.0.0.1), release build."
    );

    let mut results: Vec<(BenchConfig, Option<BenchResult>)> = Vec::new();
    for cfg in matrix {
        print_header(&cfg);
        match harness.run(&cfg).await {
            Ok(r) => {
                print_result(&r);
                results.push((cfg, Some(r)));
            }
            Err(e) => {
                print_failure(&*e);
                results.push((cfg, None));
            }
        }
    }

    print_summary(&results);
    Ok(())
}
