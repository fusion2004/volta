//! Define the "plain" format style for list commands.

use semver::Version;

use volta_core::style::tool_version;

use super::{Node, Package, PackageManager, Source, Toolchain};

pub(super) fn format(toolchain: &Toolchain) -> Option<String> {
    let (runtimes, package_managers, packages) = match toolchain {
        Toolchain::Node(runtimes) => (describe_runtimes(&runtimes), None, None),
        Toolchain::PackageManagers(package_managers) => {
            (None, describe_package_managers(&package_managers), None)
        }
        Toolchain::Packages(packages) => (None, None, describe_packages(&packages)),
        Toolchain::Tool {
            name,
            host_packages,
        } => (None, None, Some(describe_tool_set(name, host_packages))),
        Toolchain::Active {
            runtime,
            package_manager,
            packages,
        } => (
            runtime
                .as_ref()
                .and_then(|r| describe_runtimes(&[r.clone()])),
            package_manager
                .as_ref()
                .and_then(|p| describe_package_managers(&[p.clone()])),
            describe_packages(&packages),
        ),
        Toolchain::All {
            runtimes,
            package_managers,
            packages,
        } => (
            describe_runtimes(&runtimes),
            describe_package_managers(&package_managers),
            describe_packages(&packages),
        ),
    };

    match (runtimes, package_managers, packages) {
        (Some(runtimes), Some(package_managers), Some(packages)) => {
            Some(format!("{}\n{}\n{}", runtimes, package_managers, packages))
        }
        (Some(runtimes), Some(package_managers), None) => {
            Some(format!("{}\n{}", runtimes, package_managers))
        }
        (Some(runtimes), None, Some(packages)) => Some(format!("{}\n{}", runtimes, packages)),
        (Some(runtimes), None, None) => Some(format!("{}", runtimes)),
        (None, Some(package_managers), Some(packages)) => {
            Some(format!("{}\n{}", package_managers, packages))
        }
        (None, Some(package_managers), None) => Some(format!("{}", package_managers)),
        (None, None, Some(packages)) => Some(format!("{}", packages)),
        (None, None, None) => None,
    }
}

fn describe_runtimes(runtimes: &[Node]) -> Option<String> {
    if runtimes.is_empty() {
        None
    } else {
        Some(
            runtimes
                .iter()
                .map(|runtime| display_node(&runtime.source, &runtime.version))
                .collect::<Vec<String>>()
                .join("\n"),
        )
    }
}

fn describe_package_managers(package_managers: &[PackageManager]) -> Option<String> {
    if package_managers.is_empty() {
        None
    } else {
        Some(
            package_managers
                .iter()
                .map(|package_manager| display_package_manager(&package_manager))
                .collect::<Vec<String>>()
                .join("\n"),
        )
    }
}

fn describe_packages(packages: &[Package]) -> Option<String> {
    if packages.is_empty() {
        None
    } else {
        Some(
            packages
                .iter()
                .map(display_package)
                .collect::<Vec<String>>()
                .join("\n"),
        )
    }
}

fn describe_tool_set(name: &str, hosts: &[Package]) -> String {
    hosts
        .iter()
        .filter_map(|package| display_tool(name, package))
        .collect::<Vec<String>>()
        .join("\n")
}

fn display_node(source: &Source, version: &Version) -> String {
    format!("runtime {}{}", tool_version("node", version), source)
}

fn display_package_manager(package_manager: &PackageManager) -> String {
    format!(
        "package-manager {}{}",
        tool_version(&package_manager.kind, &package_manager.version),
        package_manager.source
    )
}

fn package_source(package: &Package) -> String {
    match package {
        Package::Default { .. } => String::from(" (default)"),
        Package::Project { path, .. } => format!(" (current @ {})", path.display()),
        Package::Fetched(..) => String::new(),
    }
}

fn display_package(package: &Package) -> String {
    match package {
        Package::Default {
            details,
            node,
            tools,
            ..
        }
        | Package::Project {
            details,
            node,
            tools,
            ..
        } => {
            let tools = match tools.len() {
                0 => String::from(" "),
                _ => format!(" {} ", tools.join(", ")),
            };

            format!(
                "package {} /{}/ {} {}{}",
                tool_version(&details.name, &details.version),
                tools,
                tool_version("node", &node),
                // Should be updated when we support installing with custom package_managers,
                // whether Yarn or non-built-in versions of npm
                "npm@built-in",
                package_source(&package)
            )
        }
        Package::Fetched(details) => format!(
            "package {} (fetched)",
            tool_version(&details.name, &details.version)
        ),
    }
}

fn display_tool(name: &str, host: &Package) -> Option<String> {
    match host {
        Package::Default { details, node, .. } | Package::Project { details, node, .. } => {
            Some(format!(
                "tool {} / {} / {} {}{}",
                name,
                tool_version(&details.name, &details.version),
                tool_version("node", &node),
                "npm@built-in",
                package_source(&host)
            ))
        }
        Package::Fetched(..) => None,
    }
}

// These tests are organized by way of the *item* being printed, unlike in the
// `human` module, because the formatting is consistent across command formats.
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use lazy_static::lazy_static;
    use semver::Version;

    use crate::command::list::PackageDetails;

    lazy_static! {
        static ref NODE_VERSION: Version = Version::from((12, 4, 0));
        static ref TYPESCRIPT_VERSION: Version = Version::from((3, 4, 1));
        static ref YARN_VERSION: Version = Version::from((1, 16, 0));
        static ref PROJECT_PATH: PathBuf = PathBuf::from("/a/b/c");
    }

    mod node {
        use super::super::*;
        use super::*;

        #[test]
        fn default() {
            let source = Source::Default;
            assert_eq!(
                display_node(&source, &NODE_VERSION).as_str(),
                "runtime node@12.4.0 (default)"
            );
        }

        #[test]
        fn project() {
            let source = Source::Project(PROJECT_PATH.clone());
            assert_eq!(
                display_node(&source, &NODE_VERSION).as_str(),
                "runtime node@12.4.0 (current @ /a/b/c)"
            );
        }

        #[test]
        fn installed_not_set() {
            let source = Source::None;
            assert_eq!(
                display_node(&source, &NODE_VERSION).as_str(),
                "runtime node@12.4.0"
            );
        }
    }

    mod yarn {
        use super::super::*;
        use super::*;
        use crate::command::list::*;

        #[test]
        fn default() {
            assert_eq!(
                display_package_manager(&PackageManager {
                    kind: PackageManagerKind::Yarn,
                    source: Source::Default,
                    version: YARN_VERSION.clone(),
                })
                .as_str(),
                "package-manager yarn@1.16.0 (default)"
            );
        }

        #[test]
        fn project() {
            assert_eq!(
                display_package_manager(&PackageManager {
                    kind: PackageManagerKind::Yarn,
                    source: Source::Project(PROJECT_PATH.clone()),
                    version: YARN_VERSION.clone()
                })
                .as_str(),
                "package-manager yarn@1.16.0 (current @ /a/b/c)"
            );
        }

        #[test]
        fn installed_not_set() {
            assert_eq!(
                display_package_manager(&PackageManager {
                    kind: PackageManagerKind::Yarn,
                    source: Source::None,
                    version: YARN_VERSION.clone()
                })
                .as_str(),
                "package-manager yarn@1.16.0"
            );
        }
    }

    mod package {
        use super::super::*;
        use super::*;

        #[test]
        fn single_default() {
            assert_eq!(
                describe_packages(&[Package::Default {
                    details: PackageDetails {
                        name: "typescript".into(),
                        version: TYPESCRIPT_VERSION.clone(),
                    },
                    node: NODE_VERSION.clone(),
                    tools: vec!["tsc".into(), "tsserver".into()]
                }])
                .expect("Should always return a `String` if given a non-empty set")
                .as_str(),
                "package typescript@3.4.1 / tsc, tsserver / node@12.4.0 npm@built-in (default)"
            );
        }

        #[test]
        fn single_project() {
            assert_eq!(
                describe_packages(&[Package::Project {
                    details: PackageDetails {
                        name: "typescript".into(),
                        version: TYPESCRIPT_VERSION.clone(),
                    },
                    path: PROJECT_PATH.clone(),
                    node: NODE_VERSION.clone(),
                    tools: vec!["tsc".into(), "tsserver".into()]
                }])
                .expect("Should always return a `String` if given a non-empty set")
                .as_str(),
                "package typescript@3.4.1 / tsc, tsserver / node@12.4.0 npm@built-in (current @ /a/b/c)"
            );
        }

        #[test]
        fn mixed() {
            assert_eq!(
                describe_packages(&[
                    Package::Project {
                        details: PackageDetails {
                            name: "typescript".into(),
                            version: TYPESCRIPT_VERSION.clone(),
                        },
                        path: PROJECT_PATH.clone(),
                        node: NODE_VERSION.clone(),
                        tools: vec!["tsc".into(), "tsserver".into()]
                    },
                    Package::Default {
                        details: PackageDetails {
                            name: "ember-cli".into(),
                            version: Version::from((3, 10, 0)),
                        },
                        node: NODE_VERSION.clone(),
                        tools: vec!["ember".into()],
                    },
                    Package::Fetched(PackageDetails {
                        name: "create-react-app".into(),
                        version: Version::from((1, 0, 0)),
                    })
                ])
                .expect("Should always return a `String` if given a non-empty set")
                .as_str(),
                "package typescript@3.4.1 / tsc, tsserver / node@12.4.0 npm@built-in (current @ /a/b/c)\n\
                 package ember-cli@3.10.0 / ember / node@12.4.0 npm@built-in (default)\n\
                 package create-react-app@1.0.0 (fetched)"
            );
        }

        #[test]
        fn installed_not_set() {
            assert_eq!(
                describe_packages(&[Package::Fetched(PackageDetails {
                    name: "typescript".into(),
                    version: TYPESCRIPT_VERSION.clone(),
                })])
                .expect("Should always return a `String` if given a non-empty set")
                .as_str(),
                "package typescript@3.4.1 (fetched)"
            );
        }
    }

    mod tool {
        use super::super::*;
        use super::*;

        #[test]
        fn default() {
            assert_eq!(
                display_tool(
                    "tsc",
                    &Package::Default {
                        details: PackageDetails {
                            name: "typescript".into(),
                            version: TYPESCRIPT_VERSION.clone(),
                        },
                        node: NODE_VERSION.clone(),
                        tools: vec!["tsc".into(), "tsserver".into()],
                    }
                )
                .expect("should always return `Some` for `Default`")
                .as_str(),
                "tool tsc / typescript@3.4.1 / node@12.4.0 npm@built-in (default)"
            );
        }

        #[test]
        fn project() {
            assert_eq!(
                display_tool(
                    "tsc",
                    &Package::Project {
                        details: PackageDetails {
                            name: "typescript".into(),
                            version: TYPESCRIPT_VERSION.clone(),
                        },
                        path: PROJECT_PATH.clone(),
                        node: NODE_VERSION.clone(),
                        tools: vec!["tsc".into(), "tsserver".into()],
                    }
                )
                .expect("should always return `Some` for `Project`")
                .as_str(),
                "tool tsc / typescript@3.4.1 / node@12.4.0 npm@built-in (current @ /a/b/c)"
            );
        }

        #[test]
        fn fetched() {
            assert_eq!(
                display_tool(
                    "tsc",
                    &Package::Fetched(PackageDetails {
                        name: "typescript".into(),
                        version: TYPESCRIPT_VERSION.clone()
                    })
                ),
                None
            );
        }
    }

    mod toolchain {
        use super::super::*;
        use super::*;
        use crate::command::list::{Node, PackageManager, PackageManagerKind, Toolchain};

        #[test]
        fn full() {
            assert_eq!(
                format(&Toolchain::All {
                    runtimes: vec![
                        Node {
                            source: Source::Default,
                            version: NODE_VERSION.clone()
                        },
                        Node {
                            source: Source::None,
                            version: Version::from((8, 2, 4))
                        }
                    ],
                    package_managers: vec![
                        PackageManager {
                            kind: PackageManagerKind::Yarn,
                            source: Source::Project(PROJECT_PATH.clone()),
                            version: YARN_VERSION.clone()
                        },
                        PackageManager {
                            kind: PackageManagerKind::Yarn,
                            source: Source::Default,
                            version: Version::from((1, 17, 0))
                        }
                    ],
                    packages: vec![
                        Package::Default {
                            details: PackageDetails {
                                name: "ember-cli".into(),
                                version: Version::from((3, 10, 2)),
                            },
                            node: NODE_VERSION.clone(),
                            tools: vec!["ember".into()]
                        },
                        Package::Project {
                            details: PackageDetails {
                                name: "ember-cli".into(),
                                version: Version::from((3, 8, 1)),
                            },
                            path: PROJECT_PATH.clone(),
                            node: NODE_VERSION.clone(),
                            tools: vec!["ember".into()]
                        },
                        Package::Default {
                            details: PackageDetails {
                                name: "typescript".into(),
                                version: TYPESCRIPT_VERSION.clone(),
                            },
                            node: NODE_VERSION.clone(),
                            tools: vec!["tsc".into(), "tsserver".into()]
                        }
                    ]
                })
                .expect("`format` with a non-empty toolchain returns `Some`")
                .as_str(),
                "runtime node@12.4.0 (default)\n\
                 runtime node@8.2.4\n\
                 package-manager yarn@1.16.0 (current @ /a/b/c)\n\
                 package-manager yarn@1.17.0 (default)\n\
                 package ember-cli@3.10.2 / ember / node@12.4.0 npm@built-in (default)\n\
                 package ember-cli@3.8.1 / ember / node@12.4.0 npm@built-in (current @ /a/b/c)\n\
                 package typescript@3.4.1 / tsc, tsserver / node@12.4.0 npm@built-in (default)"
            )
        }
    }
}
