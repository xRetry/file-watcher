use std::time::Duration;
use std::path::Path;
use std::fs;

use file_watcher::{run_watcher, Config, CommandConfig};

struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        std::fs::remove_dir_all("tests/files").unwrap();
        std::fs::remove_dir_all("tests/ignore").unwrap();
    }
}

#[test]
fn test_fn() {
    let _cleanup = Cleanup;

    fs::create_dir_all(Path::new("tests/files")).unwrap();
    fs::create_dir_all(Path::new("tests/ignore")).unwrap();

    fs::write("tests/files/a", "0").unwrap();
    fs::write("tests/ignore/a", "1").unwrap();

    let config = Config{
        path: None,
        exclude: Some(vec!["tests/ignore".to_string()]),
        command: vec![
            CommandConfig{
                regex: "a$".to_string(),
                cmd: Some("sh -c 'cat {file_dir}/{file} > tests/files/b'".to_string()),
                chain: None,
            }
        ],
    };

    std::thread::spawn(move || {
        run_watcher(config, None).unwrap();
    });

    std::thread::sleep(Duration::from_secs(1));

    std::fs::write("tests/files/a", "3").unwrap();
    std::fs::write("tests/ignore/a", "4").unwrap();

    std::thread::sleep(Duration::from_secs(1));

    let b_content = std::fs::read_to_string("tests/files/b").unwrap();
    assert_eq!("3", b_content);

}
