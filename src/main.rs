mod memory_system;
mod replacement_policies;

use memory_system::CacheSystem;
use std::io::{self, BufRead};
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        eprintln!("Usage: cachesim policy cache_size cache_lines associativity");
        process::exit(1);
    }

    let policy = &args[1];
    let cache_size: u64 = args[2].parse().expect("invalid cache size");
    let cache_lines: u64 = args[3].parse().expect("invalid cache lines");
    let associativity: u64 = args[4].parse().expect("invalid associativity");

    let mut cache = CacheSystem::new(cache_size, cache_lines, associativity, policy);

    let mut malformed_lines: Vec<(usize, String)> = vec![];

    for (input_line_num, line) in io::stdin().lock().lines().map_while(Result::ok).enumerate() {
        let line = line.trim().to_string();
        if line.is_empty() {
            println!("Skipping empty line.");
            continue;
        }


        let (rw, addr) = match parse_trace(&line) {
            Ok(res) => res,
            Err(e) => {
                println!("Error parsing trace line: {e}");
                malformed_lines.push((input_line_num+1, line));
                continue
            }
        };
        //println!("{} at 0x{addr:x}", if rw == 'R' { "read" } else { "write" });

        if let Err(e) = cache.access(addr, rw) {
            eprintln!("{e}");
            process::exit(1);
        }
    }

    println!("\nMALFORMED LINES:");
    malformed_lines.iter().for_each(|(i, s)| println!("  {i} {s}"));

    cache.print_stats();
}

fn parse_trace(line: &str) -> Result<(char, u64), String> {
    let Some((rw, hex)) = line.split_once(' ') else { return Err(format!("Malformed trace line: \"{line}\"")) };
    let Some(rw) = rw.chars().next() else { return Err(format!("empty R/W field: \"{line}\"")) };
    let Ok(addr) = u64::from_str_radix(hex.trim_start_matches("0x"), 16) else { return Err(format!("Invalid hex address: \"{line}\"")) };
    Ok((rw, addr))
}