use file_watcher::{run_watcher, Config, CommandConfig};

#[test]
fn test_fn() {
    let config = Config{
        path: Some("tests/files/a".to_string()),
        exclude: None,
        command: vec![
            CommandConfig{
                regex: "files/a".to_string(),
                cmd: Some("cat a > {file_dir}/b".to_string()),
                chain: None,
            }
        ],
    };

    let handle = std::thread::spawn(|| {
        //run_watcher(config);
    });

    assert!(false);
}
