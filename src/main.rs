mod memory_system;
mod replacement_policies;
mod prefetchers;

use memory_system::CacheSystem;
use std::io::{self, BufRead, Read};
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 7 {
        eprintln!("Usage: cachesim policy cache_size cache_lines associativity");
        process::exit(1);
    }

    let policy = &args[1];
    let cache_size: u64 = args[2].parse().expect("invalid cache size");
    let cache_lines: u64 = args[3].parse().expect("invalid cache lines");
    let associativity: u64 = args[4].parse().expect("invalid associativity");
    let prefetch_strategy: &str = &args[5];
    let prefetch_amount: u64 = args[6].parse().expect("Invalid prefetch amount");

    let mut cache = CacheSystem::new(cache_size, cache_lines, associativity, policy, prefetch_strategy, prefetch_amount);

    /////////////////////////////////////////////////////////////////////////////////////////////
    // Herein lies some ai generate code to emulate the C implementation's stdin parsing behavior.
    // I didn't want to manually figure out how to make my rust code exactly match the faulty C code.
    /////////////////////////////////////////////////////////////////////////////////////////////
    let mut rw: u8;
    let mut address: u32 = 0;

    // Use a buffered reader to allow byte-by-byte inspection
    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin.lock());

    loop {
        let mut buf = [0u8; 1];
        if reader.read_exact(&mut buf).is_err() {
            break; 
        }
        rw = buf[0];

        consume_whitespace(&mut reader);
        
        // Try to read hex digits.
        let hex_digits = consume_hex_digits(&mut reader);
        if !hex_digits.is_empty() {
            address = parse_hex_wrapped(&hex_digits);
        }

        consume_whitespace(&mut reader);

        let rw_char = rw as char;
        
        //println!("{} at 0x{:x}", op_name, address);
        if let Err(e) = cache.access(address as u64, rw_char, false) {
            eprintln!("{e}");
            process::exit(1);
        }
    }

    cache.print_stats();
}

/// Corrected: No longer tries to return the result of .consume()
fn consume_whitespace<R: BufRead>(reader: &mut R) {
    loop {
        let (consumed, halt, is_whitespace) = {
            let Ok(available) = reader.fill_buf() else { return };
            if available.is_empty() { 
                (0, true, false) 
            } else {
                let mut count = 0;
                let mut found_non_whitespace = false;
                for &b in available {
                    if (b as char).is_ascii_whitespace() {
                        count += 1;
                    } else {
                        found_non_whitespace = true;
                        break;
                    }
                }
                (count, found_non_whitespace, true)
            }
        };

        reader.consume(consumed);
        if halt || !is_whitespace { break; }
    }
}

/// Corrected: Explicitly returns the String after consuming the bytes
fn consume_hex_digits<R: BufRead>(reader: &mut R) -> String {
    let mut s = String::new();
    loop {
        let (consumed, halt) = {
            let Ok(available) = reader.fill_buf() else { return s };
            if available.is_empty() {
                (0, true)
            } else {
                let mut count = 0;
                for &b in available {
                    let c = b as char;
                    // Match hex digits or the 'x' in '0x'
                    if c.is_ascii_hexdigit() || c == 'x' || c == 'X' {
                        s.push(c);
                        count += 1;
                    } else {
                        // Stop if we hit a non-hex character
                        reader.consume(count);
                        return s; 
                    }
                }
                (count, false)
            }
        };
        reader.consume(consumed);
        if halt { break; }
    }
    s
}

fn parse_hex_wrapped(s: &str) -> u32 {
    let clean_s = s.trim_start_matches("0x").trim_start_matches("0X");
    let mut val: u32 = 0;
    for c in clean_s.chars() {
        if let Some(digit) = c.to_digit(16) {
            val = val.wrapping_shl(4).wrapping_add(digit as u32);
        }
    }
    val
}