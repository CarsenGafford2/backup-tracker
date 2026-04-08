mod backup_engine;
mod file_hasher;
mod local_tracker;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        println!("No arguments provided");
        return;
    }

    if args[0] == "-v" && args.len() == 1 {
        println!("Backup Tracker Version 0.0.3");
    } else if args[0] == "-track" && args.len() == 3 {
        let backup_engine = backup_engine::BackupEngine::new();
        let tracker = local_tracker::LocalTracker::new();
        let path_to_track = std::path::Path::new(&args[1]);

        if path_to_track.is_dir() {
            tracker.track_folder(&args[1]);
        } else if path_to_track.is_file() {
            tracker.track_file(&args[1]);
        }

        backup_engine.backup(&args[1], &args[2]);
        println!("File/Folder tracked and backed up successfully");
    } else if args[0] == "-update" && args.len() == 1 {
        // TODO: Implement update functionality.
    }
}
