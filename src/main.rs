use std::io::{self, Read, Write, BufRead};
use std::fs::File;
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use num_cpus;

//const JSON_FILE_PATH: &str = "test.json";

fn main() -> io::Result<()> {
    let start_time = Instant::now(); // Record start time
    let mut input_filename = String::new();

    println!("Enter the input file name:");
    io::stdin().lock().read_line(&mut input_filename)?;

    let input_source = Arc::new(Mutex::new(File::open(input_filename.trim())?));
    let output_source = Arc::new(Mutex::new(File::create("output.json")?));

    let mut handles = vec![];
    let mut total_semicolons = 0; // Variable to track total semicolons encountered

    for _ in 0..num_cpus::get() {
        let input_source_clone = Arc::clone(&input_source);
        let output_source_clone = Arc::clone(&output_source);

        let handle = thread::spawn(move || {
            let mut buffer: [u8; 5120] = [0; 5120]; // Read and write buffer of size 4KB
            let mut semicolon_count = 0; // Variable to track semicolons encountered in this chunk

            loop {
                let bytes_read = {
                    let mut input = input_source_clone.lock().unwrap();
                    input.read(&mut buffer).expect("Error reading from input")
                };

                if bytes_read == 0 {
                    break;
                }

                let (corrected_chunk, count) = replace_chars(&buffer[..bytes_read]);
                semicolon_count += count; // Update semicolons count for this chunk

                {
                    let mut output = output_source_clone.lock().unwrap();
                    output.write_all(&corrected_chunk).expect("Error writing to output");
                }
            }

            semicolon_count // Return semicolons encountered in this chunk
        });

        handles.push(handle);
    }

    for handle in handles {
        total_semicolons += handle.join().expect("Thread panicked");
    }

    let end_time = Instant::now(); // Record end time
    let runtime = end_time.duration_since(start_time); // Calculate runtime

    println!("Total semicolons encountered: {}", total_semicolons);
    println!("Runtime: {:?}", runtime);

    Ok(())
}

fn replace_chars(chunk: &[u8]) -> (Vec<u8>, usize) {
    let mut corrected_chunk = Vec::with_capacity(chunk.len());
    let mut semicolon_count = 0;

    for &byte in chunk {
        if byte == b';' {
            corrected_chunk.push(b':');
            semicolon_count += 1;
        } else {
            corrected_chunk.push(byte);
        }
    }

    (corrected_chunk, semicolon_count)
}
