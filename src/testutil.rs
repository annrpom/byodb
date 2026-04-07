/// A minimal datadriven test framework modelled after CockroachDB/Pebble's
/// `datadriven` package.
///
/// Test files contain one or more records separated by blank lines:
///
/// ```text
/// # optional comment
/// command [arg1 arg2 ...]
/// optional input line 1
/// optional input line 2
/// ----
/// expected output
/// ```
///
/// [`run_test`] parses the file, calls the supplied handler for each record,
/// and panics if the handler's output does not match the expected output.
///
/// Set the `REWRITE` environment variable to overwrite expected output
/// in-place instead of asserting — same as Pebble's `-rewrite` flag:
///
/// ```sh
/// REWRITE=1 cargo test
/// ```

/// A single parsed test record.
pub struct Record {
    pub cmd: String,
    pub args: Vec<String>,
    /// Lines between the command line and `----`.
    pub input: String,
}

/// Parses `path` and runs each record through `handler`.
///
/// Panics on any mismatch between actual and expected output, unless
/// `REWRITE=1` is set in the environment, in which case the file is
/// rewritten with the actual output.
pub fn run_test(path: &std::path::Path, handler: impl Fn(&Record) -> String) {
    let contents = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));

    let records = parse(&contents);
    let rewrite = std::env::var("REWRITE").is_ok();
    let mut rewritten = String::new();
    let mut failed = false;

    for (record, expected) in &records {
        let actual = handler(record);

        if rewrite {
            rewritten.push_str(&record.cmd);
            if !record.args.is_empty() {
                rewritten.push(' ');
                rewritten.push_str(&record.args.join(" "));
            }
            rewritten.push('\n');
            if !record.input.is_empty() {
                rewritten.push_str(&record.input);
                rewritten.push('\n');
            }
            rewritten.push_str("----\n");
            rewritten.push_str(actual.trim_end());
            rewritten.push_str("\n\n");
        } else if actual.trim_end() != expected.trim_end() {
            eprintln!(
                "FAIL {}:{}\n  expected: {:?}\n  actual:   {:?}",
                path.display(),
                record.cmd,
                expected.trim_end(),
                actual.trim_end(),
            );
            failed = true;
        }
    }

    if rewrite {
        std::fs::write(path, rewritten.trim_end().to_owned() + "\n")
            .unwrap_or_else(|e| panic!("failed to rewrite {}: {e}", path.display()));
    } else if failed {
        panic!("datadriven test failed: {}", path.display());
    }
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

fn parse(contents: &str) -> Vec<(Record, String)> {
    let mut records = Vec::new();
    let mut lines = contents.lines().peekable();

    while let Some(line) = lines.next() {
        // Skip blank lines and comments.
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }

        // First non-blank, non-comment line is the command.
        let mut tokens = line.split_whitespace();
        let cmd = tokens.next().expect("non-empty line has no tokens").to_owned();
        let args: Vec<String> = tokens.map(str::to_owned).collect();

        // Accumulate input lines until `----`.
        let mut input_lines: Vec<&str> = Vec::new();
        loop {
            match lines.next() {
                Some("----") => break,
                Some(l) => input_lines.push(l),
                None => panic!("record `{cmd}` is missing `----`"),
            }
        }
        let input = input_lines.join("\n");

        // Accumulate expected output until a blank line or EOF.
        let mut expected_lines: Vec<&str> = Vec::new();
        loop {
            match lines.peek() {
                Some(&l) if l.trim().is_empty() => break,
                Some(_) => expected_lines.push(lines.next().unwrap()),
                None => break,
            }
        }
        let expected = expected_lines.join("\n");

        records.push((Record { cmd, args, input }, expected));
    }

    records
}
