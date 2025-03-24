use std::time::Instant;

use rsmdf::{mdf::MDFFile, mdf::MDF};

fn main() {
    let mdf = MDF::new("./example_files/ASAP2_Demo_V171_deflate.mf4");

    // mdf.list_data_groups();
    mdf.list_channels();

    let start = Instant::now();
    
    // Only try to read channels if there are any
    if !mdf.channels.is_empty() {
        for channel in &mdf.channels {
            let signal = mdf.read_channel(&channel);
            println!("Channel: {}", channel.name);
            println!("  Comment: {}", signal.comment);
            match signal.max_time() {
                Some(max_time) => println!("  Max Time: {}", max_time),
                None => println!("  Max Time: No data available"),
            }
            println!();  // Add a blank line between channels for better readability
        }
    } else {
        println!("No channels found in the MDF file");
    }

    println!("Took: {:?}", start.elapsed());
}
