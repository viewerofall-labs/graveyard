use compute_pi::compute_pi_str;
use std::io;

fn main() {
    println!("How many decimal places of Pi do you want to generate?");

    // 1. Get user input
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input.");
    
    // Parse the input into a number
    let requested_chars: usize = input.trim().parse().unwrap_or(0);

    if requested_chars == 0 {
        println!("Please enter a valid number greater than 0.");
        return;
    }

    // 2. Generate Pi using the crate!
    // Note: The number you pass in dictates the number of *decimal places* generated.
    let pi_string = compute_pi_str(requested_chars);

    // Print the result!
    println!("\nHere is Pi to {} decimal places:\n{}", requested_chars, pi_string);
}
