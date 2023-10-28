use std::time::Duration;

use file_watcher::{run_watcher, Config, CommandConfig};

#[test]
fn test_fn() {
    let config = Config{
        path: None,
        exclude: None,
        command: vec![
            CommandConfig{
                regex: "a$".to_string(),
                cmd: Some("sh -c 'cat {file_dir}/{file} > {file_dir}/b'".to_string()),
                chain: None,
            }
        ],
    };

    std::thread::spawn(move || {
        run_watcher(config, None).unwrap();
    });

    std::thread::sleep(Duration::from_secs(1));

    std::fs::write("tests/files/a", "3").unwrap();

    std::thread::sleep(Duration::from_secs(3));

    let b_content = std::fs::read_to_string("tests/files/b").unwrap();
    assert_eq!("3", b_content);
}
