use byodb::testutil;
use datatest_stable::harness;

fn run(path: &std::path::Path) -> datatest_stable::Result<()> {
    let prefix = std::path::Path::new("testdata");
    let rel = path.strip_prefix(prefix).unwrap_or(path);
    let module = rel.iter().next().and_then(|s| s.to_str()).unwrap_or("");

    match module {
        "kv" => testutil::run_test(path, kv::run),
        other => panic!("unknown test module: {other}"),
    }

    Ok(())
}

mod kv {
    use byodb::testutil::Record;

    pub fn run(r: &Record) -> String {
        match r.cmd.as_str() {
            // Commands will be added as the kv implementation grows.
            other => panic!("unknown command: {other}"),
        }
    }
}

harness! {
    { test = run, root = "testdata", pattern = r".*" },
}
