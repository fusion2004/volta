use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use envoy;
use serde_json;

use volta_core::fs::symlink_file;
use volta_core::tool::{NODE_DISTRO_ARCH, NODE_DISTRO_OS};

use test_support::{self, ok_or_panic, paths, paths::PathExt, process::ProcessBuilder};

#[derive(PartialEq, Clone)]
pub struct FileBuilder {
    path: PathBuf,
    contents: String,
}

impl FileBuilder {
    pub fn new(path: PathBuf, contents: &str) -> FileBuilder {
        FileBuilder {
            path,
            contents: contents.to_string(),
        }
    }

    pub fn build(&self) {
        self.dirname().mkdir_p();

        let mut file = File::create(&self.path)
            .unwrap_or_else(|e| panic!("could not create file {}: {}", self.path.display(), e));

        ok_or_panic! { file.write_all(self.contents.as_bytes()) };
    }

    fn dirname(&self) -> &Path {
        self.path.parent().unwrap()
    }
}

#[must_use]
pub struct TempProjectBuilder {
    root: TempProject,
    files: Vec<FileBuilder>,
}

impl TempProjectBuilder {
    /// Root of the project, ex: `/path/to/cargo/target/smoke_test/t0/foo`
    pub fn root(&self) -> PathBuf {
        self.root.root()
    }

    pub fn new(root: PathBuf) -> TempProjectBuilder {
        TempProjectBuilder {
            root: TempProject {
                root: root.clone(),
                path: OsString::new(),
            },
            files: vec![],
        }
    }

    /// Set the package.json for the temporary project (chainable)
    pub fn package_json(mut self, contents: &str) -> Self {
        let package_file = package_json_file(self.root());
        self.files.push(FileBuilder::new(package_file, contents));
        self
    }

    /// Create the project
    pub fn build(mut self) -> TempProject {
        // First, clean the temporary project directory if it already exists
        self.rm_root();

        // Create the empty directory
        self.root.root().mkdir_p();

        // make sure these directories exist and are empty
        node_cache_dir(self.root()).ensure_empty();
        volta_bin_dir(self.root()).ensure_empty();
        node_inventory_dir(self.root()).ensure_empty();
        yarn_inventory_dir(self.root()).ensure_empty();
        package_inventory_dir(self.root()).ensure_empty();
        node_image_root_dir(self.root()).ensure_empty();
        yarn_image_root_dir(self.root()).ensure_empty();
        package_image_root_dir(self.root()).ensure_empty();
        user_toolchain_dir(self.root()).ensure_empty();
        volta_tmp_dir(self.root()).ensure_empty();

        // and these files do not exist
        volta_file(self.root()).rm();
        shim_executable(self.root()).rm();
        user_hooks_file(self.root()).rm();
        user_platform_file(self.root()).rm();

        // create symlinks to shim executable for node, yarn, npm, and packages
        ok_or_panic!(symlink_file(shim_exe(), self.root.node_exe()));
        ok_or_panic!(symlink_file(shim_exe(), self.root.yarn_exe()));
        ok_or_panic!(symlink_file(shim_exe(), self.root.npm_exe()));

        ok_or_panic!(symlink_file(
            shim_exe(),
            shim_executable(self.root())
        ));

        // write files
        for file_builder in self.files {
            file_builder.build();
        }

        // prepend Volta bin dir to the PATH
        let current_path = envoy::path().expect("Could not get current PATH");
        let new_path = current_path.split();
        self.root.path = new_path
            .prefix_entry(volta_bin_dir(self.root.root()))
            .join()
            .expect("Failed to join paths");

        let TempProjectBuilder { root, .. } = self;
        root
    }

    fn rm_root(&self) {
        self.root.root().rm_rf()
    }
}

// files and dirs in the temporary project

fn home_dir(root: PathBuf) -> PathBuf {
    root.join("home")
}
fn volta_home(root: PathBuf) -> PathBuf {
    home_dir(root).join(".volta")
}
fn volta_file(root: PathBuf) -> PathBuf {
    volta_home(root).join("volta")
}
fn shim_executable(root: PathBuf) -> PathBuf {
    volta_home(root).join("shim")
}
fn user_hooks_file(root: PathBuf) -> PathBuf {
    volta_home(root).join("hooks.json")
}
fn volta_tmp_dir(root: PathBuf) -> PathBuf {
    volta_home(root).join("tmp")
}
fn volta_bin_dir(root: PathBuf) -> PathBuf {
    volta_home(root).join("bin")
}
fn volta_tools_dir(root: PathBuf) -> PathBuf {
    volta_home(root).join("tools")
}
fn inventory_dir(root: PathBuf) -> PathBuf {
    volta_tools_dir(root).join("inventory")
}
fn user_toolchain_dir(root: PathBuf) -> PathBuf {
    volta_tools_dir(root).join("user")
}
fn user_dir(root: PathBuf) -> PathBuf {
    volta_tools_dir(root).join("user")
}
fn image_dir(root: PathBuf) -> PathBuf {
    volta_tools_dir(root).join("image")
}
fn node_image_root_dir(root: PathBuf) -> PathBuf {
    image_dir(root).join("node")
}
fn node_image_dir(node: &str, npm: &str, root: PathBuf) -> PathBuf {
    node_image_root_dir(root).join(node).join(npm)
}
fn node_image_bin_dir(node: &str, npm: &str, root: PathBuf) -> PathBuf {
    node_image_dir(node, npm, root).join("bin")
}
fn yarn_image_root_dir(root: PathBuf) -> PathBuf {
    image_dir(root).join("yarn")
}
fn yarn_image_dir(version: &str, root: PathBuf) -> PathBuf {
    yarn_image_root_dir(root).join(version)
}
fn package_image_root_dir(root: PathBuf) -> PathBuf {
    image_dir(root).join("packages")
}
fn node_inventory_dir(root: PathBuf) -> PathBuf {
    inventory_dir(root).join("node")
}
fn yarn_inventory_dir(root: PathBuf) -> PathBuf {
    inventory_dir(root).join("yarn")
}
fn package_inventory_dir(root: PathBuf) -> PathBuf {
    inventory_dir(root).join("packages")
}
fn package_distro_file(name: &str, version: &str, root: PathBuf) -> PathBuf {
    package_inventory_dir(root).join(package_distro_file_name(name, version))
}
fn package_distro_shasum(name: &str, version: &str, root: PathBuf) -> PathBuf {
    package_inventory_dir(root).join(package_shasum_file_name(name, version))
}
fn cache_dir(root: PathBuf) -> PathBuf {
    volta_home(root).join("cache")
}
fn node_cache_dir(root: PathBuf) -> PathBuf {
    cache_dir(root).join("node")
}
fn package_json_file(mut root: PathBuf) -> PathBuf {
    root.push("package.json");
    root
}
fn shim_file(name: &str, root: PathBuf) -> PathBuf {
    volta_bin_dir(root).join(format!("{}{}", name, env::consts::EXE_SUFFIX))
}
fn package_image_dir(name: &str, version: &str, root: PathBuf) -> PathBuf {
    image_dir(root).join("packages").join(name).join(version)
}
fn user_platform_file(root: PathBuf) -> PathBuf {
    user_dir(root).join("platform.json")
}
pub fn node_distro_file_name(version: &str) -> String {
    format!("node-v{}-{}-{}.tar.gz", version, NODE_DISTRO_OS, NODE_DISTRO_ARCH)
}
fn yarn_distro_file_name(version: &str) -> String {
    format!("yarn-v{}.tar.gz", version)
}
fn package_distro_file_name(name: &str, version: &str) -> String {
    format!("{}-{}.tgz", name, version)
}
fn package_shasum_file_name(name: &str, version: &str) -> String {
    format!("{}-{}.shasum", name, version)
}

pub struct TempProject {
    root: PathBuf,
    path: OsString,
}

impl TempProject {
    /// Root of the project, ex: `/path/to/cargo/target/integration_test/t0/foo`
    pub fn root(&self) -> PathBuf {
        self.root.clone()
    }

    /// Create a `ProcessBuilder` to run a program in the project.
    /// Example:
    ///         assert_that(
    ///             p.process(&p.bin("foo")),
    ///             execs().with_stdout("bar\n"),
    ///         );
    pub fn process<T: AsRef<OsStr>>(&self, program: T) -> ProcessBuilder {
        let mut p = test_support::process::process(program);
        p.cwd(self.root())
            // setup the Volta environment
            .env("PATH", &self.path)
            .env("HOME", home_dir(self.root()))
            .env("VOLTA_HOME", volta_home(self.root()))
            .env_remove("VOLTA_NODE_VERSION")
            .env_remove("MSYSTEM"); // assume cmd.exe everywhere on windows

        p
    }

    /// Create a `ProcessBuilder` to run volta.
    /// Arguments can be separated by spaces.
    /// Example:
    ///     assert_that(p.volta("use node 9.5"), execs());
    pub fn volta(&self, cmd: &str) -> ProcessBuilder {
        let mut p = self.process(&volta_exe());
        split_and_add_args(&mut p, cmd);
        p
    }

    /// Create a `ProcessBuilder` to run Node.
    pub fn node(&self, cmd: &str) -> ProcessBuilder {
        let mut p = self.process(&self.node_exe());
        split_and_add_args(&mut p, cmd);
        p
    }

    pub fn node_exe(&self) -> PathBuf {
        volta_bin_dir(self.root()).join(format!("node{}", env::consts::EXE_SUFFIX))
    }

    /// Create a `ProcessBuilder` to run Yarn.
    pub fn yarn(&self, cmd: &str) -> ProcessBuilder {
        let mut p = self.process(&self.yarn_exe());
        split_and_add_args(&mut p, cmd);
        p
    }

    pub fn yarn_exe(&self) -> PathBuf {
        volta_bin_dir(self.root()).join(format!("yarn{}", env::consts::EXE_SUFFIX))
    }

    /// Create a `ProcessBuilder` to run Npm.
    pub fn npm(&self, cmd: &str) -> ProcessBuilder {
        let mut p = self.process(&self.npm_exe());
        split_and_add_args(&mut p, cmd);
        p
    }

    pub fn npm_exe(&self) -> PathBuf {
        volta_bin_dir(self.root()).join(format!("npm{}", env::consts::EXE_SUFFIX))
    }

    /// Create a `ProcessBuilder` to run a package executable.
    pub fn exec_shim(&self, exe: &str, cmd: &str) -> ProcessBuilder {
        let shim_file = shim_file(exe, self.root());
        let mut p = self.process(shim_file);
        split_and_add_args(&mut p, cmd);
        p
    }

    /// Verify that the input Node version has been fetched.
    pub fn node_version_is_fetched(&self, version: &str) -> bool {
        let distro_file_name = node_distro_file_name(version);
        let inventory_dir = node_inventory_dir(self.root());
        inventory_dir.join(distro_file_name).exists()
    }

    /// Verify that the input Node version has been unpacked.
    pub fn node_version_is_unpacked(&self, version: &str, npm_version: &str) -> bool {
        let unpack_dir = node_image_bin_dir(version, npm_version, self.root());
        unpack_dir.exists()
    }

    /// Verify that the input Node version has been installed.
    pub fn assert_node_version_is_installed(&self, version: &str, npm_version: &str) -> () {
        let user_platform = user_platform_file(self.root());
        let platform_contents = read_file_to_string(user_platform);
        let json_contents: serde_json::Value =
            serde_json::from_str(&platform_contents).expect("could not parse platform.json");
        assert_eq!(json_contents["node"]["runtime"], version);
        assert_eq!(json_contents["node"]["npm"], npm_version);
    }

    /// Verify that the input Yarn version has been fetched.
    pub fn yarn_version_is_fetched(&self, version: &str) -> bool {
        let distro_file_name = yarn_distro_file_name(version);
        let inventory_dir = yarn_inventory_dir(self.root());
        inventory_dir.join(distro_file_name).exists()
    }

    /// Verify that the input Yarn version has been unpacked.
    pub fn yarn_version_is_unpacked(&self, version: &str) -> bool {
        let unpack_dir = yarn_image_dir(version, self.root());
        unpack_dir.exists()
    }

    /// Verify that the input Yarn version has been installed.
    pub fn assert_yarn_version_is_installed(&self, version: &str) -> () {
        let user_platform = user_platform_file(self.root());
        let platform_contents = read_file_to_string(user_platform);
        let json_contents: serde_json::Value =
            serde_json::from_str(&platform_contents).expect("could not parse platform.json");
        assert_eq!(json_contents["yarn"], version);
    }

    /// Verify that the input Npm version has been fetched.
    pub fn npm_version_is_fetched(&self, version: &str) -> bool {
        // ISSUE(#292): This is maybe the wrong place to put npm?
        let package_file = package_distro_file("npm", version, self.root());
        let shasum_file = package_distro_shasum("npm", version, self.root());
        package_file.exists() && shasum_file.exists()
    }

    /// Verify that the input Npm version has been unpacked.
    pub fn npm_version_is_unpacked(&self, version: &str) -> bool {
        // ISSUE(#292): This is maybe the wrong place to unpack npm?
        let unpack_dir = package_image_dir("npm", version, self.root());
        unpack_dir.exists()
    }

    /// Verify that the input Npm version has been installed.
    pub fn assert_npm_version_is_installed(&self, version: &str) -> () {
        let user_platform = user_platform_file(self.root());
        let platform_contents = read_file_to_string(user_platform);
        let json_contents: serde_json::Value =
            serde_json::from_str(&platform_contents).expect("could not parse platform.json");
        assert_eq!(json_contents["node"]["npm"], version);
    }

    /// Verify that the input package version has been fetched.
    pub fn package_version_is_fetched(&self, name: &str, version: &str) -> bool {
        let package_file = package_distro_file(name, version, self.root());
        let shasum_file = package_distro_shasum(name, version, self.root());
        package_file.exists() && shasum_file.exists()
    }

    /// Verify that the input package version has been unpacked.
    pub fn package_version_is_unpacked(&self, name: &str, version: &str) -> bool {
        let unpack_dir = package_image_dir(name, version, self.root());
        unpack_dir.exists()
    }

    /// Verify that the input package version has been fetched.
    pub fn shim_exists(&self, name: &str) -> bool {
        shim_file(name, self.root()).exists()
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        self.root().rm_rf();
    }
}

// Generates a temporary project environment
pub fn temp_project() -> TempProjectBuilder {
    TempProjectBuilder::new(paths::root().join("temp-project"))
}

// Path to compiled executables
pub fn cargo_dir() -> PathBuf {
    env::var_os("CARGO_BIN_PATH")
        .map(PathBuf::from)
        .or_else(|| {
            env::current_exe().ok().map(|mut path| {
                path.pop();
                if path.ends_with("deps") {
                    path.pop();
                }
                path
            })
        })
        .unwrap_or_else(|| panic!("CARGO_BIN_PATH wasn't set. Cannot continue running test"))
}

fn volta_exe() -> PathBuf {
    cargo_dir().join(format!("volta{}", env::consts::EXE_SUFFIX))
}

fn shim_exe() -> PathBuf {
    cargo_dir().join(format!("shim{}", env::consts::EXE_SUFFIX))
}

fn split_and_add_args(p: &mut ProcessBuilder, s: &str) {
    for arg in s.split_whitespace() {
        if arg.contains('"') || arg.contains('\'') {
            panic!("shell-style argument parsing is not supported")
        }
        p.arg(arg);
    }
}

fn read_file_to_string(file_path: PathBuf) -> String {
    let mut contents = String::new();
    let mut file = ok_or_panic! { File::open(file_path) };
    ok_or_panic! { file.read_to_string(&mut contents) };
    contents
}
