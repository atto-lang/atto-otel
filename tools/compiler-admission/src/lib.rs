//! Host compiler-launch admission regression tests.
//! These are host shell/git fixture tests, not proof of guest-language semantics.

#[cfg(all(test, unix))]
mod tests {
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        path::{Path, PathBuf},
        process::{Command, Output},
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let sequence = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "ores-otel-compiler-{}-{nanos}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("create isolated fixture");
            Self { path }
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn git(source: &Path, args: &[&str]) -> String {
        let result = Command::new("git")
            .arg("-C")
            .arg(source)
            .args(args)
            .output()
            .expect("run git fixture");
        assert!(
            result.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&result.stderr)
        );
        String::from_utf8(result.stdout)
            .expect("git UTF-8 output")
            .trim()
            .to_owned()
    }

    struct Fixture {
        temp: TestDir,
        source: PathBuf,
        java: PathBuf,
        marker: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let temp = TestDir::new();
            fs::create_dir(temp.path.join("scripts")).expect("scripts directory");
            let compiler_sh =
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/compiler.sh");
            fs::copy(compiler_sh, temp.path.join("scripts/compiler.sh"))
                .expect("copy actual compiler launcher");

            let source = temp.path.join("compiler");
            fs::create_dir(&source).expect("compiler directory");
            git(&source, &["init", "-q"]);
            fs::write(source.join(".gitignore"), "target/\n").unwrap();
            fs::write(source.join("tracked"), "original").unwrap();
            git(&source, &["add", "."]);
            git(
                &source,
                &[
                    "-c",
                    "user.name=Fixture",
                    "-c",
                    "user.email=fixture@example.invalid",
                    "commit",
                    "-qm",
                    "fixture",
                ],
            );
            let pin = git(&source, &["rev-parse", "HEAD"]);
            fs::write(temp.path.join("SOURCE_REF"), pin).unwrap();
            fs::create_dir(source.join("target")).unwrap();
            fs::write(source.join("target/classpath.txt"), "fixture-classpath").unwrap();

            let marker = temp.path.join("java-called");
            let java = temp.path.join("java");
            fs::write(&java, "#!/bin/sh\ntouch \"$JAVA_MARKER\"\n").unwrap();
            let mut permissions = fs::metadata(&java).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&java, permissions).unwrap();

            Self {
                temp,
                source,
                java,
                marker,
            }
        }

        fn launch(&self, source_dir: Option<&Path>) -> Output {
            Command::new("bash")
                .arg("scripts/compiler.sh")
                .arg("test.ores")
                .current_dir(&self.temp.path)
                .env(
                    "ORESLANG_SOURCE_DIR",
                    source_dir.unwrap_or(self.source.as_path()),
                )
                .env("ORES_JAVA", &self.java)
                .env("JAVA_MARKER", &self.marker)
                .output()
                .expect("run real compiler.sh fixture")
        }

        fn rejected(&self, message: &str, source_dir: Option<&Path>) {
            let out = self.launch(source_dir);
            assert!(
                !out.status.success(),
                "launcher admitted forbidden checkout: {}",
                String::from_utf8_lossy(&out.stdout)
            );
            assert!(
                String::from_utf8_lossy(&out.stderr).contains(message),
                "missing diagnostic {message:?}: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            assert!(
                !self.marker.exists(),
                "fake Java executed before admission checks"
            );
        }
    }

    #[test]
    fn exact_clean_pin_launches() {
        let fixture = Fixture::new();
        let out = fixture.launch(None);
        assert!(
            out.status.success(),
            "clean exact pin failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(fixture.marker.exists(), "clean pin did not launch Java");
    }

    #[test]
    fn wrong_and_floating_pins_are_rejected() {
        let fixture = Fixture::new();
        fs::write(fixture.temp.path.join("SOURCE_REF"), "0".repeat(40)).unwrap();
        fixture.rejected("revision differs", None);
        fs::write(fixture.temp.path.join("SOURCE_REF"), "main").unwrap();
        fixture.rejected("exact compiler commit", None);
    }

    #[test]
    fn dirty_and_untracked_checkouts_are_rejected() {
        let fixture = Fixture::new();
        fs::write(fixture.source.join("tracked"), "developer edits").unwrap();
        fixture.rejected("must be clean", None);
        fs::write(fixture.source.join("tracked"), "original").unwrap();
        fs::write(fixture.source.join("untracked"), "extra source").unwrap();
        fixture.rejected("must be clean", None);
    }

    #[test]
    fn nested_directory_cannot_impersonate_checkout() {
        let fixture = Fixture::new();
        let child = fixture.source.join("nested");
        fs::create_dir(&child).unwrap();
        fixture.rejected("its own Git checkout", Some(&child));
    }
}
