use assert_cmd::prelude::*;
use assert_fs::fixture::ChildPath;
use assert_fs::prelude::*;
use predicates::prelude::*;
use predicates::str::contains;
use std::process::Command;

const CONFIG_1: &str = r#"
[scripts.test_cmd_1]
alias = 'test_cmd_1'
command = 'echo test_1'
tags = ['info', 'echo', 'grp_1']

[scripts.test_cmd_2]
command = 'echo test_2'
tags = ['debug', 'echo']

[scripts.test_success]
alias = 'test_success'
command = '''
#!/bin/sh
exit 0
'''

[scripts.test_fail]
alias = 'test_fail'
command = '''
#!/bin/sh
exit 1
'''

[scripts.test_exit_with_100]
alias = 'test_exit_with_100'
command = '''
#!/bin/sh
exit 100
'''

[scripts.shebang-with-args]
alias = 'shebang-with-args'
command = '''
#!/bin/sh
echo "$1--$2"
''' 

[scripts.inline-with-args]
alias = 'inline-with-args'
command = 'echo "$1--$2"'

"#;

// Tests listing all scripts
pier_test!(cli => test_list_scripts, cfg => CONFIG_1,
| _cfg: ChildPath, mut cmd: Command | {
    // TODO Add some way to verify the output other than exit code.
    cmd.arg("list");
    cmd.assert()
        .success();
});

// Tests listing alias
pier_test!(cli => test_run_shebang_with_args, cfg => CONFIG_1,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.arg("run")
    .arg("shebang-with-args")
    .arg("Hello!")
    .arg("Hi.");

    cmd.assert()
        .stdout(contains("Hello!--Hi."))
        .success();
});

pier_test!(cli => test_run_inline_with_args, cfg => CONFIG_1,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.arg("run")
    .arg("inline-with-args")
    .env("SHELL", "/bin/sh")
    .arg("Hello!")
    .arg("Hi.");

    cmd.assert()
        .stdout(contains("Hello!--Hi."))
        .success();
});

// Tests script with arguments.
pier_test!(cli => test_list_alias_scripts, cfg => CONFIG_1,
| _cfg: ChildPath, mut cmd: Command | {
    // TODO Add some way to verify the output other than exit code.
    cmd.arg("ls");
    cmd.assert()
        .success();
});

// Tests script with successful exit code
pier_test!(cli => test_run_successful, cfg => CONFIG_1,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["run", "test_success"]);
    cmd.assert()
        .success();
});

// Tests script with failing exit code
pier_test!(cli => test_run_failing, cfg => CONFIG_1,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["run", "test_fail"]);
    cmd.assert()
        .failure();
});

// Tests script with custom exit code
pier_test!(cli => test_run_custom_exit_code, cfg => CONFIG_1,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["run", "test_exit_with_100"]);
    cmd.assert()
        .failure()
    .code(100);
});

// Tests listing all scripts
pier_test!(cli => test_list_scripts_with_command_width, cfg => CONFIG_1,
| _cfg: ChildPath, mut cmd: Command | {
    // TODO Add some way to verify the output other than exit code.
    cmd.args(["list", "-c", "20"]);
    cmd.assert()
        .success();
});

// WORK IN PROGRESS
pier_test!(basic => test_config_initialization,
| te: crate::common::TestEnv | {
    let cfg = te.dir.child("pier.toml");

    let mut cmd = Command::cargo_bin("pier").expect("Failed to set cargo binary pier");
    cmd.current_dir(te.dir.path());
    cmd.args(["-c", cfg.path().to_str().unwrap(), "init"]);
    cmd.assert().success();

        cfg.assert(predicate::path::exists());
});

// Tests listing all aliases
pier_test!(cli => test_list_aliases, cfg => CONFIG_1,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["list", "-q"]);
    cmd.assert()
        .success()
        .stdout(contains(trim!(r#"
        test_cmd_1
        test_cmd_2
    "#)).trim()
    );
});

// Tests listing all aliases with matching tag
pier_test!(cli => test_list_alias_with_matching_tag, cfg => CONFIG_1,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["list", "-q", "-t", "info"]);
    cmd.assert()
        .success()
        .stdout(contains(trim!(r#"
        test_cmd_1
    "#)).trim()
    );
});

// Tests that no aliases is listed when no tag is matched
pier_test!(cli => test_list_without_tag_match, cfg => CONFIG_1,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["list", "-q", "-t", "bad_tag"]);
    cmd.assert()
        .success()
        .stdout(contains("").trim()
    );
});

// Tests adding a script
pier_test!(cli => test_add_script, cfg => CONFIG_1,
| cfg: ChildPath, mut cmd: Command | {
    cmd.args(["add", r#"echo test_3"#, "-a", "test_cmd_3", "-d", "Test Description."]);
    cmd.assert().success();

    cfg.assert(contains(trim!(r#"
        [scripts.test_cmd_3]
        command = "echo test_3"
        description = "Test Description."
    "#)).trim()
    );
});

// Tests adding a script with forcing
pier_test!(cli => test_add_script_force_script, cfg => CONFIG_1,
| cfg: ChildPath, mut cmd: Command | {
    cmd.args(["add", r#"echo test_3"#, "-a", "test_cmd_1", "-d", "Test Description.", "-f"]);
    cmd.assert().success();

    cfg.assert(contains(trim!(r#"
        [scripts.test_cmd_1]
        command = "echo test_3"
        description = "Test Description."
    "#)).trim()
    );
});

// Tests copying a script
pier_test!(cli => test_copy_script, cfg => CONFIG_1,
| cfg: ChildPath, mut cmd: Command | {
    cmd.args(["copy", "test_cmd_1", "test_cmd_4"]);
    cmd.assert().success();

    cfg.assert(contains(trim!(r#"
            [scripts.test_cmd_1]
            command = "echo test_1"
        "#)).trim()
    );

    cfg.assert(contains(trim!(r#"
            [scripts.test_cmd_4]
            command = "echo test_1"
        "#)).trim()
    );
});

// Tests moving a script
pier_test!(cli => test_move_script, cfg => CONFIG_1,
| cfg: ChildPath, mut cmd: Command | {
    cmd.args(["move", "test_cmd_1", "test_cmd_4"]);
    cmd.assert().success();

    cfg.assert(contains(trim!(r#"
            [scripts.test_cmd_4]
            command = "echo test_1"
        "#)).trim()
    );
    cfg.assert(contains(trim!(r#"
            [scripts.test_cmd_1]
            alias = "test_cmd_1"
            command = "echo test_1"
        "#)).trim().not()
    );
});

// Tests moving a script with forcing
pier_test!(cli => test_move_with_force_script, cfg => CONFIG_1,
| cfg: ChildPath, mut cmd: Command | {
    cmd.args(["move", "test_cmd_1", "test_cmd_2", "-f"]);
    cmd.assert().success();

    cfg.assert(contains(trim!(r#"
            [scripts.test_cmd_2]
            command = "echo test_1"
        "#)).trim()
    );
    cfg.assert(contains(trim!(r#"
            [scripts.test_cmd_1]
            command = "echo test_1"
        "#)).trim().not()
    );
});

// Tests removing a script
pier_test!(cli => test_remove_script, cfg => CONFIG_1,
| cfg: ChildPath, mut cmd: Command | {
    cmd.args(["remove", "test_cmd_1"]);
    cmd.assert().success();
    cfg.assert(contains(trim!(r#"
        [scripts.test_cmd_1]
        alias = 'test_cmd_1'
        command = 'echo test_1' 
        "#)).trim().not()
    );

});

// Tests show a script
pier_test!(cli => test_show_script, cfg => r#"
[scripts.test_show]
alias = 'test_show'
command = '''
#!/bin/sh
for line in l1 l2 l3 l4; do
echo "$line"
done
'''
"#,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["show", "test_show"]);
    cmd.assert()
        .success()
        .stdout(contains(trim!(r#"
            for line in l1 l2 l3 l4; do
                echo "$line"
            done"#
        )));
});

// Tests running a very basic script
pier_test!(cli => test_run_script, cfg => CONFIG_1,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["run", "test_cmd_1"]);
    cmd.assert()
        .success()
        .stdout(contains("test_1"));
});

// Tests running a script with a pipe in it
pier_test!(cli => test_run_script_pipe, cfg => r#"
[scripts.test_pipe]
alias = "test_pipe"
command = '''
#!/bin/sh
echo "TEST_RUN_SCRIPT_PIPE_OUTPUT" | tr '[:upper:]' '[:lower:]'
'''
"#, | _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["run", "test_pipe"]);
    cmd.assert()
        .success()
        .stdout(contains("test_run_script_pipe_output"));
});

// Tests running a script which requires waiting for the process to end.
pier_test!(cli => test_run_script_non_blocking, cfg => r#"
[scripts.test_while_loop]
alias = "test_while_loop"
command = '''
#!/bin/sh
echo "1\\n2\\n3" | while read i; do echo "line read $i"; done
'''
"#, | _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["run", "test_while_loop"]);
    cmd.assert()
        .success();
});

// Tests running a script with a &&
pier_test!(cli => test_run_script_and, cfg => r#"
[scripts.test_and]
alias = "test_and"
command = '''
#!/bin/sh
echo "test-x-1" && echo "test-x-2"
'''
"#, | _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["run", "test_and"]);
    cmd.assert()
        .success()
        .stdout(contains("test-x-1\ntest-x-2"));
});

// Tests that stderr is printed when the command outputs it.
pier_test!(cli => test_stderr_output, cfg => r#"
[scripts.write_to_stderr]
alias = "write_to_stderr"
command = '''
#!/bin/sh
echo "WRITE THIS TO STDERR" >&2
'''
"#, | _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["run", "write_to_stderr"]);
    cmd.assert()
        .success()
        .stderr(contains("WRITE THIS TO STDERR"));
});

// Tests that shebangs work.
pier_test!(cli => test_run_with_shebang, cfg => r#"
[scripts.test_shebang]
alias = "test_shebang"
command = '''
#!/usr/bin/env python3
print("Running python script with shebang!")
'''
"#, | _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["run", "test_shebang"]);
    cmd.assert()
        .success()
        .stdout(contains("Running python script with shebang!"));
});

// Tests that default interpreter works
pier_test!(cli => test_run_with_interpreter, cfg => r#"
[default]
interpreter = ["python3", "-c"]

[scripts.test_default_interpreter]
alias = "test_default_interpreter"
command = '''
print("Running python script with interpreter!")
'''
"#, | _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["run", "test_default_interpreter"]);
    cmd.assert()
        .success()
        .stdout(contains("Running python script with interpreter!"));
});

// Tests that `pier run` subcommand forwards dash-prefixed args
pier_test!(cli => test_run_subcommand_forwards_dash_args, cfg => CONFIG_1,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.arg("run")
    .arg("inline-with-args")
    .env("SHELL", "/bin/sh")
    .arg("--hello")
    .arg("--world");

    cmd.assert()
        .stdout(contains("--hello----world"))
        .success();
});

// Regression: shorthand mode still forwards dash-prefixed args
pier_test!(cli => test_shorthand_forwards_dash_args, cfg => CONFIG_1,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.arg("inline-with-args")
    .env("SHELL", "/bin/sh")
    .arg("--hello")
    .arg("--world");

    cmd.assert()
        .stdout(contains("--hello----world"))
        .success();
});

// === Dry-run tests ===

pier_test!(cli => test_dry_run_shows_command, cfg => CONFIG_1,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["run", "--dry-run", "test_cmd_1"]);
    cmd.assert()
        .success()
        .stdout(contains("echo test_1"));
});

pier_test!(cli => test_dry_run_shows_interpreter, cfg => r#"
[default]
interpreter = ["python3", "-c"]

[scripts.my_script]
alias = "my_script"
command = 'print("hello")'
"#,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["run", "--dry-run", "my_script"]);
    cmd.assert()
        .success()
        .stdout(contains("Interpreter:").and(contains("python3")))
        .stdout(contains("Command:").and(contains("print")));
});

pier_test!(cli => test_dry_run_no_side_effect, cfg => r#"
[scripts.touch_test]
alias = "touch_test"
command = 'touch /tmp/pier_dry_run_test_marker'
"#,
| _cfg: ChildPath, mut cmd: Command | {
    // Clean up marker if it exists from a previous run
    let _ = std::fs::remove_file("/tmp/pier_dry_run_test_marker");

    cmd.env("SHELL", "/bin/sh")
       .args(["run", "--dry-run", "touch_test"]);
    cmd.assert().success();

    assert!(!std::path::Path::new("/tmp/pier_dry_run_test_marker").exists(),
        "Dry-run should not execute the command");
});

pier_test!(cli => test_dry_run_shorthand, cfg => CONFIG_1,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["--dry-run", "test_cmd_1"]);
    cmd.assert()
        .success()
        .stdout(contains("echo test_1"));
});

pier_test!(cli => test_dry_run_shows_shebang_info, cfg => r#"
[scripts.my_shebang]
alias = "my_shebang"
command = '''
#!/usr/bin/env python3
print("hello from shebang")
'''
"#,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["run", "--dry-run", "my_shebang"]);
    cmd.assert()
        .success()
        .stdout(contains("Mode:").and(contains("shebang")))
        .stdout(contains("print(\"hello from shebang\")"));
});

// === Shell completions tests ===

pier_test!(basic => test_completions_bash, | _te: crate::common::TestEnv | {
    let mut cmd = Command::cargo_bin("pier").expect("Failed to set cargo binary pier");
    cmd.args(["completions", "bash"]);
    cmd.assert()
        .success()
        .stdout(contains("pier"));
});

pier_test!(basic => test_completions_zsh, | _te: crate::common::TestEnv | {
    let mut cmd = Command::cargo_bin("pier").expect("Failed to set cargo binary pier");
    cmd.args(["completions", "zsh"]);
    cmd.assert()
        .success()
        .stdout(contains("pier"));
});

pier_test!(basic => test_completions_fish, | _te: crate::common::TestEnv | {
    let mut cmd = Command::cargo_bin("pier").expect("Failed to set cargo binary pier");
    cmd.args(["completions", "fish"]);
    cmd.assert()
        .success()
        .stdout(contains("pier"));
});

// === Import / Export tests ===

const CONFIG_EXPORT: &str = r#"
[scripts.export_cmd_1]
alias = 'export_cmd_1'
command = 'echo export_1'
tags = ['info', 'echo']

[scripts.export_cmd_2]
command = 'echo export_2'
tags = ['debug']

[scripts.export_cmd_3]
command = 'echo export_3'
"#;

pier_test!(cli => test_export_all, cfg => CONFIG_EXPORT,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.arg("export");
    cmd.assert()
        .success()
        .stdout(contains("[scripts.export_cmd_1]"))
        .stdout(contains("[scripts.export_cmd_2]"))
        .stdout(contains("[scripts.export_cmd_3]"));
});

pier_test!(cli => test_export_single_alias, cfg => CONFIG_EXPORT,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["export", "--alias", "export_cmd_1"]);
    cmd.assert()
        .success()
        .stdout(contains("[scripts.export_cmd_1]"))
        .stdout(contains("[scripts.export_cmd_2]").not())
        .stdout(contains("[scripts.export_cmd_3]").not());
});

pier_test!(cli => test_export_by_tag, cfg => CONFIG_EXPORT,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["export", "--tag", "echo"]);
    cmd.assert()
        .success()
        .stdout(contains("[scripts.export_cmd_1]"))
        .stdout(contains("[scripts.export_cmd_2]").not())
        .stdout(contains("[scripts.export_cmd_3]").not());
});

pier_test!(cli => test_export_nonexistent_alias, cfg => CONFIG_EXPORT,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["export", "--alias", "nonexistent"]);
    cmd.assert()
        .failure();
});

pier_test!(cli => test_import_scripts, cfg => r#"
[scripts.original]
command = 'echo original'
"#,
| cfg: ChildPath, mut cmd: Command | {
    let import_content = r#"
[scripts.imported]
command = 'echo imported'
"#;
    let import_file = cfg.path().parent().unwrap().join("import.toml");
    std::fs::write(&import_file, import_content).unwrap();

    cmd.args(["import", import_file.to_str().unwrap()]);
    cmd.assert().success();

    cfg.assert(contains("echo imported"));
    cfg.assert(contains("echo original"));
});

pier_test!(cli => test_import_conflict_fails, cfg => CONFIG_EXPORT,
| cfg: ChildPath, mut cmd: Command | {
    let import_content = r#"
[scripts.export_cmd_1]
command = 'echo conflict'
"#;
    let import_file = cfg.path().parent().unwrap().join("import.toml");
    std::fs::write(&import_file, import_content).unwrap();

    cmd.args(["import", import_file.to_str().unwrap()]);
    cmd.assert().failure();
});

pier_test!(cli => test_import_force_overwrites, cfg => CONFIG_EXPORT,
| cfg: ChildPath, mut cmd: Command | {
    let import_content = r#"
[scripts.export_cmd_1]
command = 'echo overwritten'
"#;
    let import_file = cfg.path().parent().unwrap().join("import.toml");
    std::fs::write(&import_file, import_content).unwrap();

    cmd.args(["import", "--force", import_file.to_str().unwrap()]);
    cmd.assert().success();

    cfg.assert(contains("echo overwritten"));
});

pier_test!(cli => test_import_export_roundtrip, cfg => CONFIG_EXPORT,
| cfg: ChildPath, mut cmd: Command | {
    // Export
    let output = cmd.arg("export").output().expect("Failed to run export");
    assert!(output.status.success());
    let exported = String::from_utf8(output.stdout).unwrap();

    // Import into a fresh config
    let fresh_config = cfg.path().parent().unwrap().join("fresh.toml");
    std::fs::write(&fresh_config, "").unwrap();

    let mut import_cmd = Command::cargo_bin("pier").expect("Failed to set cargo binary pier");
    let import_file = cfg.path().parent().unwrap().join("exported.toml");
    std::fs::write(&import_file, &exported).unwrap();

    import_cmd.args(["-c", fresh_config.to_str().unwrap(), "import", import_file.to_str().unwrap()]);
    import_cmd.assert().success();

    // Verify the fresh config has the scripts
    let fresh_content = std::fs::read_to_string(&fresh_config).unwrap();
    assert!(fresh_content.contains("export_cmd_1"));
    assert!(fresh_content.contains("export_cmd_2"));
    assert!(fresh_content.contains("export_cmd_3"));
});

// === Namespace tests ===

const CONFIG_NS: &str = r#"
[scripts."dev:start"]
command = 'echo starting dev'
tags = ['echo']

[scripts."dev:stop"]
command = 'echo stopping dev'

[scripts."prod:deploy"]
command = 'echo deploying prod'
tags = ['echo']
"#;

pier_test!(cli => test_namespace_run, cfg => CONFIG_NS,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.env("SHELL", "/bin/sh")
       .args(["run", "dev:start"]);
    cmd.assert()
        .success()
        .stdout(contains("starting dev"));
});

pier_test!(cli => test_namespace_list_filter, cfg => CONFIG_NS,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["list", "-n", "dev"]);
    cmd.assert()
        .success()
        .stdout(contains("dev:start"))
        .stdout(contains("dev:stop"))
        .stdout(contains("prod:deploy").not());
});

pier_test!(cli => test_namespace_list_aliases_filter, cfg => CONFIG_NS,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["list", "-q", "-n", "dev"]);
    cmd.assert()
        .success()
        .stdout(contains("dev:start"))
        .stdout(contains("dev:stop"))
        .stdout(contains("prod:deploy").not());
});

pier_test!(cli => test_namespace_and_tag_combined, cfg => CONFIG_NS,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["list", "-q", "-n", "dev", "-t", "echo"]);
    cmd.assert()
        .success()
        .stdout(contains("dev:start"))
        .stdout(contains("dev:stop").not())
        .stdout(contains("prod:deploy").not());
});

// === Tag management tests ===

const CONFIG_TAGS: &str = r#"
[scripts.tagged]
command = 'echo tagged'
tags = ['info', 'echo']

[scripts.untagged]
command = 'echo untagged'
"#;

pier_test!(cli => test_tag_list, cfg => CONFIG_TAGS,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["tag", "list"]);
    cmd.assert()
        .success()
        .stdout(contains("echo"))
        .stdout(contains("info"));
});

pier_test!(cli => test_tag_add, cfg => CONFIG_TAGS,
| cfg: ChildPath, mut cmd: Command | {
    cmd.args(["tag", "add", "untagged", "new_tag", "another"]);
    cmd.assert().success();

    cfg.assert(contains("new_tag"));
    cfg.assert(contains("another"));
});

pier_test!(cli => test_tag_add_no_duplicates, cfg => CONFIG_TAGS,
| cfg: ChildPath, mut cmd: Command | {
    cmd.args(["tag", "add", "tagged", "info", "new_tag"]);
    cmd.assert().success();

    let content = std::fs::read_to_string(cfg.path()).unwrap();
    assert_eq!(content.matches("info").count(), 1, "info should not be duplicated");
    assert!(content.contains("new_tag"));
});

pier_test!(cli => test_tag_remove, cfg => r#"
[scripts.tagged]
command = 'echo tagged'
tags = ['info', 'removeme']
"#,
| cfg: ChildPath, mut cmd: Command | {
    cmd.args(["tag", "rm", "tagged", "removeme"]);
    cmd.assert().success();

    let content = std::fs::read_to_string(cfg.path()).unwrap();
    assert!(content.contains("info"), "remaining tag should stay");
    assert!(!content.contains("removeme"), "removed tag should be gone");
});

pier_test!(cli => test_tag_remove_all_clears_field, cfg => r#"
[scripts.one_tag]
command = 'echo one'
tags = ['only']
"#,
| cfg: ChildPath, mut cmd: Command | {
    cmd.args(["tag", "rm", "one_tag", "only"]);
    cmd.assert().success();

    let content = std::fs::read_to_string(cfg.path()).unwrap();
    assert!(!content.contains("tags"), "tags field should be removed when empty");
});

pier_test!(cli => test_tag_add_nonexistent_alias, cfg => CONFIG_TAGS,
| _cfg: ChildPath, mut cmd: Command | {
    cmd.args(["tag", "add", "nonexistent", "foo"]);
    cmd.assert().failure();
});
