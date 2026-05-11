use datatest_stable::harness;

fn run(path: &std::path::Path) -> datatest_stable::Result<()> {
    let prefix = std::path::Path::new("testdata");
    let rel = path.strip_prefix(prefix).unwrap_or(path);
    let module = rel.iter().next().and_then(|s| s.to_str()).unwrap_or("");

    match module {
        "kv" => kv::run_file(path),
        other => panic!("unknown test module: {other}"),
    }

    Ok(())
}

mod kv {
    use byodb::kv::KV;
    use byodb::testutil::Record;
    use std::cell::RefCell;
    use std::path::{Path, PathBuf};

    pub fn run_file(path: &Path) {
        let dir = std::env::temp_dir()
            .join("byodb-test")
            .join(path.file_stem().unwrap_or_default());
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let db_path: PathBuf = dir.join("db");

        let kv: RefCell<Option<KV>> = RefCell::new(None);
        byodb::testutil::run_test(path, |r| run(r, &kv, &db_path));
    }

    fn run(r: &Record, kv: &RefCell<Option<KV>>, db_path: &Path) -> String {
        match r.cmd.as_str() {
            "open" => {
                *kv.borrow_mut() = Some(KV::open(db_path).unwrap());
                "ok".to_string()
            }
            "set" => {
                kv.borrow_mut()
                    .as_mut()
                    .unwrap()
                    .set(&r.args[0], &r.args[1])
                    .unwrap();
                "ok".to_string()
            }
            "get" => match kv.borrow().as_ref().unwrap().get(&r.args[0]) {
                Some(v) => String::from_utf8(v).unwrap(),
                None => "(not found)".to_string(),
            },
            "delete" => {
                kv.borrow_mut()
                    .as_mut()
                    .unwrap()
                    .delete(&r.args[0])
                    .unwrap();
                "ok".to_string()
            }
            other => panic!("unknown command: {other}"),
        }
    }
}

harness! {
    { test = run, root = "testdata", pattern = r".*" },
}
