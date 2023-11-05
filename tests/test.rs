use std::time::Duration;
use std::path::Path;
use std::fs;

use file_watcher::{run_watcher, Config, CommandConfig};

struct Cleanup {
    path: String,
}

impl Drop for Cleanup {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.path).unwrap();
    }
}

#[test]
fn test_simple() {

    fs::create_dir_all(Path::new("tests/files/test_simple")).unwrap();
    let _cleanup = Cleanup{path: "tests/files/test_simple".into()};

    fs::write("tests/files/test_simple/a", "a").unwrap();
    fs::write("tests/files/test_simple/b", "b").unwrap();
    fs::write("tests/files/test_simple/out", "0").unwrap();

    let config = Config{
        path: None,
        exclude: None,
        command: vec![
            CommandConfig{
                regex: "test_simple/a$".to_string(),
                cmd: Some("sh -c 'cat {file_dir}/{file} >> tests/files/test_simple/out'".to_string()),
                chain: None,
            },
            CommandConfig{
                regex: "test_simple/b$".to_string(),
                cmd: Some("sh -c 'cat {file_dir}/{file} >> tests/files/test_simple/out'".to_string()),
                chain: None,
            },
        ],
    };

    std::thread::spawn(move || {
        run_watcher(config).unwrap();
    });

    std::thread::sleep(Duration::from_secs(1));

    std::fs::write("tests/files/test_simple/a", "3").unwrap();
    std::fs::write("tests/files/test_simple/b", "4").unwrap();

    std::thread::sleep(Duration::from_secs(1));

    let b_content = std::fs::read_to_string("tests/files/test_simple/out").unwrap();
    assert_eq!("034", b_content);

    drop(_cleanup);
}

#[test]
fn test_chain() {

    fs::create_dir_all(Path::new("tests/files/test_chain")).unwrap();
    let _cleanup = Cleanup{path: "tests/files/test_chain".into()};

    fs::write("tests/files/test_chain/a", "a").unwrap();
    fs::write("tests/files/test_chain/out", "0").unwrap();

    let config = Config{
        path: None,
        exclude: None,
        command: vec![
            CommandConfig{
                regex: "test_chain/a$".to_string(),
                cmd: None,
                chain: Some(vec![
                    "sh -c 'echo 1 >> tests/files/test_chain/out'".to_string(),
                    "sh -c 'echo 2 >> tests/files/test_chain/out'".to_string(),
                ]),
            }
        ],
    };

    std::thread::spawn(move || {
        run_watcher(config).unwrap();
    });

    std::thread::sleep(Duration::from_secs(1));

    std::fs::write("tests/files/test_chain/a", "b").unwrap();

    std::thread::sleep(Duration::from_secs(1));

    let b_content = std::fs::read_to_string("tests/files/test_chain/out").unwrap();
    assert_eq!("01\n2\n", b_content);

    drop(_cleanup);
}
