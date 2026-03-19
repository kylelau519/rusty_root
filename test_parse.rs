use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

fn main() {
    let path = "/Users/kylelau519/Programming/rusty_root/rusty_root_io/testfiles/wzqcd_mc20a.root";
    let file = File::open(path).expect("Failed to open ROOT file");
    let mut reader = std::io::BufReader::new(file);

    // From test_read_all_streamer_info, we need to extract the TList data
    // Let's just run cargo test and capture the log
}
