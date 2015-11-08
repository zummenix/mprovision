extern crate mprovision;

fn main() {
    if let Ok(directory) = mprovision::directory() {
        if let Ok(files) = mprovision::files(directory) {
            println!("Found {} files.", files.count());
        }
    }
}
