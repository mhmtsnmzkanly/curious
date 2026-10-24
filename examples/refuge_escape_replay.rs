//! Zero-tick streaming audit of an existing refuge-lab JSONL file.
//! This example has no simulation-world import and never advances a World.
#[path = "support/refuge_escape_state.rs"]
mod state_audit;

use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
};

const MAX_FILE_BYTES: u64 = 128 * 1024 * 1024;
const MAX_LINE_BYTES: usize = 8 * 1024 * 1024;

fn next_bounded_line<R: BufRead>(reader: &mut R, line: &mut Vec<u8>) -> io::Result<Option<usize>> {
    line.clear();
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return if line.is_empty() {
                Ok(None)
            } else {
                Ok(Some(line.len()))
            };
        }
        let take = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |newline| newline + 1);
        if line.len().saturating_add(take) > MAX_LINE_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "JSONL line exceeds the 8 MiB cap",
            ));
        }
        let finished = available[take - 1] == b'\n';
        line.extend_from_slice(&available[..take]);
        reader.consume(take);
        if finished {
            return Ok(Some(line.len()));
        }
    }
}

fn run(path: PathBuf) -> Result<Value, Box<dyn std::error::Error>> {
    let metadata = fs::metadata(&path)?;
    if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
        return Err(format!("input must be a file no larger than {MAX_FILE_BYTES} bytes").into());
    }
    if path.extension().and_then(|extension| extension.to_str()) != Some("jsonl") {
        return Err("input path must have a .jsonl extension".into());
    }

    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut line = Vec::with_capacity(64 * 1024);
    let mut bytes_read = 0u64;
    let mut jsonl_line_count = 0usize;
    let mut tick_record_count = 0usize;
    let mut expected_physical_ticks = None;
    let mut focal_id = None;
    let mut header_seen = false;
    let mut summary_seen = false;
    let mut branch_end_arms = BTreeSet::<String>::new();
    let mut per_arm_record_counts = BTreeMap::<String, usize>::new();
    let mut paired_states = BTreeMap::<usize, (Option<Vec<u8>>, Option<Vec<u8>>)>::new();
    let mut pair_stored_states = 0usize;
    let mut canonical_bytes_retained = 0usize;
    let mut max_state_bytes = 0usize;

    while let Some(line_bytes) = next_bounded_line(&mut reader, &mut line)? {
        bytes_read = bytes_read.saturating_add(line_bytes as u64);
        if bytes_read > MAX_FILE_BYTES {
            return Err("streamed input exceeded the 128 MiB file cap".into());
        }
        jsonl_line_count += 1;
        let record: Value = serde_json::from_slice(&line)?;
        let kind = record["record_type"]
            .as_str()
            .ok_or("JSONL record is missing record_type")?;
        match kind {
            "header" => {
                if header_seen {
                    return Err("input contains multiple headers".into());
                }
                header_seen = true;
                expected_physical_ticks = record["max_physical_ticks"].as_u64();
                focal_id = record["focal_id"].as_u64().map(|id| id as usize);
            }
            "prefix_tick" => {
                tick_record_count += 1;
                *per_arm_record_counts
                    .entry(record["arm"].as_str().unwrap_or("unknown").to_owned())
                    .or_default() += 1;
            }
            "suffix_tick" => {
                tick_record_count += 1;
                let arm = record["arm"].as_str().unwrap_or("unknown");
                *per_arm_record_counts.entry(arm.to_owned()).or_default() += 1;
                let state = &record["full_pre_world_state_for_choice"];
                let suffix_tick = record["suffix_tick"].as_u64().map(|tick| tick as usize);
                let relevant_arm = match arm {
                    "trained_online" => Some(0),
                    "trained_frozen" => Some(1),
                    _ => None,
                };
                if let (Some(slot), Some(tick), Some(state)) =
                    (relevant_arm, suffix_tick, state.as_object().map(|_| state))
                {
                    let focal_id = focal_id.ok_or("header is missing focal_id")?;
                    let bytes =
                        state_audit::normalized_pre_world_state_bytes(state.clone(), focal_id);
                    max_state_bytes = max_state_bytes.max(bytes.len());
                    canonical_bytes_retained = canonical_bytes_retained.saturating_add(bytes.len());
                    pair_stored_states += 1;
                    let pair = paired_states.entry(tick).or_default();
                    let destination = if slot == 0 { &mut pair.0 } else { &mut pair.1 };
                    if destination.replace(bytes).is_some() {
                        return Err(
                            format!("duplicate paired state for arm {arm} tick {tick}").into()
                        );
                    }
                }
            }
            "branch_end" => {
                let arm = record["arm"].as_str().ok_or("branch_end is missing arm")?;
                if !branch_end_arms.insert(arm.to_owned()) {
                    return Err(format!("duplicate branch_end for arm {arm}").into());
                }
            }
            "summary" => {
                if summary_seen {
                    return Err("input contains multiple Summary records".into());
                }
                summary_seen = true;
            }
            _ => {}
        }
    }

    if !header_seen {
        return Err("input has no header".into());
    }
    let mut paired_ticks = 0usize;
    let mut identical_normalized_state_pairs = 0usize;
    for (online, frozen) in paired_states.values() {
        if let (Some(online), Some(frozen)) = (online, frozen) {
            paired_ticks += 1;
            identical_normalized_state_pairs += usize::from(online == frozen);
        }
    }
    let run_complete = expected_physical_ticks == Some(270)
        && summary_seen
        && tick_record_count == 270
        && per_arm_record_counts.get("shared_prefix_online") == Some(&30)
        && per_arm_record_counts.get("trained_online") == Some(&60)
        && per_arm_record_counts.get("trained_frozen") == Some(&60)
        && per_arm_record_counts.get("naive_online") == Some(&60)
        && per_arm_record_counts.get("rule_based") == Some(&60)
        && branch_end_arms
            == [
                "naive_online",
                "rule_based",
                "trained_frozen",
                "trained_online",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect();

    Ok(json!({
        "mode": "zero_tick_replay",
        "input_bytes": bytes_read,
        "jsonl_line_count": jsonl_line_count,
        "record_count": tick_record_count,
        "expected_physical_ticks": expected_physical_ticks,
        "pair_stored_states": pair_stored_states,
        "canonical_bytes_retained": canonical_bytes_retained,
        "max_state_bytes": max_state_bytes,
        "paired_ticks": paired_ticks,
        "identical_normalized_state_pairs": identical_normalized_state_pairs,
        "per_arm_record_counts": per_arm_record_counts,
        "world_ticks_executed": 0,
        "run_complete": run_complete,
        "summary_record_seen": summary_seen,
        "branch_end_arms": branch_end_arms,
        "interpretation": "offline representation/memory validation only; not an independent physics audit or experiment result"
    }))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = env::args_os().skip(1);
    let path = arguments
        .next()
        .ok_or("usage: refuge_escape_replay <existing-raw.jsonl>")?;
    if arguments.next().is_some() || path.to_string_lossy().starts_with('-') {
        return Err("expected exactly one explicit existing .jsonl path".into());
    }
    let summary = run(PathBuf::from(path))?;
    serde_json::to_writer(io::stdout().lock(), &summary)?;
    io::stdout().lock().write_all(b"\n")?;
    Ok(())
}
