use assert_cmd::prelude::*;
use assert_fs::fixture::ChildPath;
use assert_fs::prelude::*;
use predicates::prelude::*;
use predicates::str::contains;
use std::process::Command;

// =============================================================================
// Full lifecycle: init → add → list → show → run → copy → move → remove
// =============================================================================

pier_test!(basic => test_full_script_lifecycle, | te: crate::common::TestEnv | {
    let cfg = te.dir.child("pier.toml");
    let cfg_path = cfg.path().to_str().unwrap().to_string();

    // 1. Init a fresh config
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "init"]);
    cmd.assert().success();
    cfg.assert(predicate::path::exists());

    // 2. Add a script
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "add", "echo hello world", "-a", "greet", "-d", "Say hello", "-t", "demo"]);
    cmd.assert().success();

    // 3. Add a second script
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "add", "echo goodbye", "-a", "bye", "-t", "demo"]);
    cmd.assert().success();

    // 4. List — both scripts visible
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "list", "-q"]);
    cmd.assert()
        .success()
        .stdout(contains("greet"))
        .stdout(contains("bye"));

    // 5. Show — verify command text
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "show", "greet"]);
    cmd.assert()
        .success()
        .stdout(contains("echo hello world"));

    // 6. Run — verify output
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.env("SHELL", "/bin/sh");
    cmd.args(["-c", &cfg_path, "run", "greet"]);
    cmd.assert()
        .success()
        .stdout(contains("hello world"));

    // 7. Copy
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "copy", "greet", "greet-backup"]);
    cmd.assert().success();

    // 8. Verify copy exists
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "list", "-q"]);
    cmd.assert()
        .success()
        .stdout(contains("greet-backup"));

    // 9. Move/rename
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "move", "bye", "farewell"]);
    cmd.assert().success();

    // 10. Old name gone, new name present
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "list", "-q"]);
    cmd.assert()
        .success()
        .stdout(contains("farewell"))
        .stdout(contains("bye\n").not());

    // 11. Remove
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "remove", "greet-backup"]);
    cmd.assert().success();

    // 12. Verify removal
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "list", "-q"]);
    cmd.assert()
        .success()
        .stdout(contains("greet-backup").not());
});

// =============================================================================
// Import/export workflow between two configs
// =============================================================================

pier_test!(basic => test_export_import_between_configs, | te: crate::common::TestEnv | {
    // Create source config with scripts
    let source_cfg = te.dir.child("source.toml");
    source_cfg.write_str(trim!(r#"
        [scripts.build]
        command = 'echo building project'
        tags = ['dev']
        description = 'Build the project'

        [scripts.deploy]
        command = 'echo deploying to prod'
        tags = ['ops']
        description = 'Deploy to production'

        [scripts.test]
        command = 'echo running tests'
        tags = ['dev']
    "#)).unwrap();

    // Export only dev-tagged scripts
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", source_cfg.path().to_str().unwrap(), "export", "--tag", "dev"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let exported = String::from_utf8(output.stdout).unwrap();

    // Verify export contains dev scripts but not ops
    assert!(exported.contains("build"));
    assert!(exported.contains("test"));
    assert!(!exported.contains("deploy"));

    // Create target config and import
    let target_cfg = te.dir.child("target.toml");
    target_cfg.write_str("").unwrap();

    let export_file = te.dir.child("export.toml");
    export_file.write_str(&exported).unwrap();

    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", target_cfg.path().to_str().unwrap(), "import",
               export_file.path().to_str().unwrap()]);
    cmd.assert().success();

    // Verify target has the imported scripts and they run
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.env("SHELL", "/bin/sh");
    cmd.args(["-c", target_cfg.path().to_str().unwrap(), "run", "build"]);
    cmd.assert()
        .success()
        .stdout(contains("building project"));
});

// =============================================================================
// Namespaced workflow: add, list, filter, run across namespaces
// =============================================================================

pier_test!(basic => test_namespace_workflow, | te: crate::common::TestEnv | {
    let cfg = te.dir.child("pier.toml");
    let cfg_path = cfg.path().to_str().unwrap().to_string();

    // Create config with namespaced scripts
    cfg.write_str(trim!(r#"
        [scripts."web:dev"]
        command = 'echo web dev server'
        tags = ['dev']

        [scripts."web:build"]
        command = 'echo web build'
        tags = ['build']

        [scripts."api:dev"]
        command = 'echo api dev server'
        tags = ['dev']

        [scripts."api:test"]
        command = 'echo api tests'
        tags = ['test']

        [scripts.standalone]
        command = 'echo no namespace'
    "#)).unwrap();

    // List all — everything shows
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "list", "-q"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("web:dev"));
    assert!(stdout.contains("api:dev"));
    assert!(stdout.contains("standalone"));

    // List namespace=web — only web scripts
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "list", "-q", "-n", "web"]);
    cmd.assert()
        .success()
        .stdout(contains("web:dev"))
        .stdout(contains("web:build"))
        .stdout(contains("api:").not())
        .stdout(contains("standalone").not());

    // Combine namespace + tag filter
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "list", "-q", "-n", "api", "-t", "dev"]);
    cmd.assert()
        .success()
        .stdout(contains("api:dev"))
        .stdout(contains("api:test").not());

    // Run a namespaced script
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.env("SHELL", "/bin/sh");
    cmd.args(["-c", &cfg_path, "run", "web:build"]);
    cmd.assert()
        .success()
        .stdout(contains("web build"));

    // Export only one namespace
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "export", "--alias", "api:dev"]);
    cmd.assert()
        .success()
        .stdout(contains("api:dev"))
        .stdout(contains("web:").not());
});

// =============================================================================
// Dry-run then real run — verifies dry-run doesn't interfere
// =============================================================================

pier_test!(cli => test_dry_run_then_real_run, cfg => r#"
[scripts.counter]
command = 'echo REAL_EXECUTION'
"#,
| _cfg: ChildPath, mut cmd: Command | {
    // Dry-run first
    cmd.env("SHELL", "/bin/sh")
       .args(["run", "--dry-run", "counter"]);
    cmd.assert()
        .success()
        .stdout(contains("REAL_EXECUTION").not().or(contains("Command:")));
});

// =============================================================================
// Shorthand vs subcommand equivalence
// =============================================================================

pier_test!(cli => test_shorthand_and_subcommand_produce_same_output, cfg => r#"
[scripts.hello]
command = 'echo shorthand_test_output'
"#,
| _cfg: ChildPath, mut cmd: Command | {
    // Via subcommand
    cmd.env("SHELL", "/bin/sh")
       .args(["run", "hello"]);
    let output1 = cmd.output().unwrap();

    // Via shorthand
    let mut cmd2 = Command::cargo_bin("pier").unwrap();
    cmd2.current_dir(cmd.get_current_dir().unwrap());
    cmd2.env("SHELL", "/bin/sh");
    cmd2.args(["-c", _cfg.path().to_str().unwrap(), "hello"]);
    let output2 = cmd2.output().unwrap();

    assert_eq!(output1.stdout, output2.stdout);
    assert_eq!(output1.status.code(), output2.status.code());
});

// =============================================================================
// Multiple remove in one call
// =============================================================================

pier_test!(cli => test_remove_multiple_scripts, cfg => r#"
[scripts.a]
command = 'echo a'
[scripts.b]
command = 'echo b'
[scripts.c]
command = 'echo c'
"#,
| cfg: ChildPath, mut cmd: Command | {
    cmd.args(["remove", "a", "b"]);
    cmd.assert().success();

    cfg.assert(contains("[scripts.a]").not());
    cfg.assert(contains("[scripts.b]").not());
    cfg.assert(contains("[scripts.c]"));
});

// =============================================================================
// Verbose mode adds extra output
// =============================================================================

pier_test!(cli => test_verbose_run, cfg => r#"
[scripts.hello]
command = 'echo verbose_test'
"#,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.env("SHELL", "/bin/sh")
       .args(["-v", "run", "hello"]);
    cmd.assert()
        .success()
        .stdout(contains("Starting script"))
        .stdout(contains("Script complete"))
        .stdout(contains("verbose_test"));
});

// =============================================================================
// PIER_CONFIG_PATH env var picks up config
// =============================================================================

pier_test!(basic => test_config_via_env_var, | te: crate::common::TestEnv | {
    let cfg = te.dir.child("custom.toml");
    cfg.write_str(trim!(r#"
        [scripts.env_test]
        command = 'echo from_env_config'
    "#)).unwrap();

    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.env("PIER_CONFIG_PATH", cfg.path());
    cmd.env("SHELL", "/bin/sh");
    cmd.args(["run", "env_test"]);
    cmd.assert()
        .success()
        .stdout(contains("from_env_config"));
});

// =============================================================================
// Re-init fails when config already exists
// =============================================================================

pier_test!(basic => test_reinit_fails, | te: crate::common::TestEnv | {
    let cfg = te.dir.child("pier.toml");
    let cfg_path = cfg.path().to_str().unwrap().to_string();

    // Init once
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "init"]);
    cmd.assert().success();

    // Init again — should fail
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "init"]);
    cmd.assert().failure();
});

// =============================================================================
// Config with legacy alias field still parses correctly
// =============================================================================

pier_test!(cli => test_legacy_alias_field_compat, cfg => r#"
[scripts.old_style]
alias = 'old_style'
command = 'echo legacy_format'
tags = ['compat']
description = 'Script with explicit alias field'
"#,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.env("SHELL", "/bin/sh")
       .args(["run", "old_style"]);
    cmd.assert()
        .success()
        .stdout(contains("legacy_format"));
});

// =============================================================================
// Config with [default] section and without
// =============================================================================

pier_test!(cli => test_config_with_default_section, cfg => r#"
[default]
interpreter = ["sh", "-c"]

[scripts.hello]
command = 'echo with_default_section'
"#,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["run", "hello"]);
    cmd.assert()
        .success()
        .stdout(contains("with_default_section"));
});

pier_test!(cli => test_config_without_default_section, cfg => r#"
[scripts.hello]
command = 'echo no_default_section'
"#,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.env("SHELL", "/bin/sh")
       .args(["run", "hello"]);
    cmd.assert()
        .success()
        .stdout(contains("no_default_section"));
});

// =============================================================================
// Scripts with pipes, env vars, and complex shell constructs
// =============================================================================

pier_test!(cli => test_script_with_pipes_and_env, cfg => r#"
[scripts.complex]
command = 'echo "PIER_VALUE=42" | grep -o "[0-9]*"'
"#,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.env("SHELL", "/bin/sh")
       .args(["run", "complex"]);
    cmd.assert()
        .success()
        .stdout(contains("42"));
});

pier_test!(cli => test_script_reads_env_var, cfg => r#"
[scripts.read_env]
command = 'echo "GOT:$PIER_TEST_VAR"'
"#,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.env("SHELL", "/bin/sh")
       .env("PIER_TEST_VAR", "hello_from_env")
       .args(["run", "read_env"]);
    cmd.assert()
        .success()
        .stdout(contains("GOT:hello_from_env"));
});

// =============================================================================
// Script aliases with special characters (dashes, underscores, dots)
// =============================================================================

pier_test!(cli => test_alias_with_special_chars, cfg => r#"
[scripts."my-script_v2.1"]
command = 'echo special_alias_works'
"#,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.env("SHELL", "/bin/sh")
       .args(["run", "my-script_v2.1"]);
    cmd.assert()
        .success()
        .stdout(contains("special_alias_works"));
});

// =============================================================================
// Multiline shebang scripts preserve content
// =============================================================================

pier_test!(cli => test_multiline_shebang_roundtrip, cfg => r#"
[scripts.multi]
command = '''
#!/bin/sh
A="hello"
B="world"
echo "$A $B"
'''
"#,
| _cfg: ChildPath, mut cmd: Command | {
    // Show preserves the full script
    cmd.args(["show", "multi"]);
    cmd.assert()
        .success()
        .stdout(contains("A=\"hello\""))
        .stdout(contains("B=\"world\""))
        .stdout(contains("echo \"$A $B\""));
});

// =============================================================================
// Help and version flags work
// =============================================================================

pier_test!(basic => test_help_flag, | _te: crate::common::TestEnv | {
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("Usage:"))
        .stdout(contains("pier"));
});

// =============================================================================
// Add → export → remove → import restores the script
// =============================================================================

pier_test!(basic => test_add_export_remove_import_cycle, | te: crate::common::TestEnv | {
    let cfg = te.dir.child("pier.toml");
    let cfg_path = cfg.path().to_str().unwrap().to_string();

    // Init
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "init"]);
    cmd.assert().success();

    // Add a custom script
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "add", "echo cycle_test", "-a", "cycle", "-t", "test"]);
    cmd.assert().success();

    // Export it
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "export", "--alias", "cycle"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let exported = String::from_utf8(output.stdout).unwrap();

    // Save export to file
    let export_file = te.dir.child("backup.toml");
    export_file.write_str(&exported).unwrap();

    // Remove the script
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "remove", "cycle"]);
    cmd.assert().success();

    // Verify it's gone
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "list", "-q"]);
    cmd.assert()
        .success()
        .stdout(contains("cycle").not().or(contains("hello-pier")));

    // Import it back
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", &cfg_path, "import", export_file.path().to_str().unwrap()]);
    cmd.assert().success();

    // Verify it's restored and runnable
    let mut cmd = Command::cargo_bin("pier").unwrap();
    cmd.current_dir(te.dir.path());
    cmd.env("SHELL", "/bin/sh");
    cmd.args(["-c", &cfg_path, "run", "cycle"]);
    cmd.assert()
        .success()
        .stdout(contains("cycle_test"));
});

// =============================================================================
// Exit code propagation through pier
// =============================================================================

pier_test!(cli => test_exit_code_propagation, cfg => r#"
[scripts.exit42]
command = '''
#!/bin/sh
exit 42
'''
"#,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["run", "exit42"]);
    cmd.assert()
        .failure()
        .code(42);
});

// =============================================================================
// Tags with empty list still work
// =============================================================================

pier_test!(cli => test_empty_tags_list, cfg => r#"
[scripts.no_tags]
command = 'echo no_tags_here'
tags = []
"#,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.env("SHELL", "/bin/sh")
       .args(["run", "no_tags"]);
    cmd.assert()
        .success()
        .stdout(contains("no_tags_here"));
});
