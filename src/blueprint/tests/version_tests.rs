//! Comprehensive tests for BlueprintVersion

use crate::blueprint::evolution::BlueprintVersion;
use crate::blueprint::tests::TestAssertions;

#[cfg(test)]
mod blueprint_version_tests {
    use super::*;

    mod version_parsing_tests {
        use super::*;

        #[test]
        fn test_basic_version_parsing() {
            let version = BlueprintVersion::from_str("1.2.3").unwrap();
            assert_eq!(version.major, 1);
            assert_eq!(version.minor, 2);
            assert_eq!(version.patch, 3);
            assert_eq!(version.pre_release, None);
            assert_eq!(version.build, None);
        }

        #[test]
        fn test_version_with_prerelease() {
            let version = BlueprintVersion::from_str("1.2.3-alpha").unwrap();
            assert_eq!(version.major, 1);
            assert_eq!(version.minor, 2);
            assert_eq!(version.patch, 3);
            assert_eq!(version.pre_release, Some("alpha".to_string()));
            assert_eq!(version.build, None);
        }

        #[test]
        fn test_version_with_build() {
            let version = BlueprintVersion::from_str("1.2.3+build123").unwrap();
            assert_eq!(version.major, 1);
            assert_eq!(version.minor, 2);
            assert_eq!(version.patch, 3);
            assert_eq!(version.pre_release, None);
            assert_eq!(version.build, Some("build123".to_string()));
        }

        #[test]
        fn test_version_with_prerelease_and_build() {
            let version = BlueprintVersion::from_str("2.0.0-alpha.1+build.456").unwrap();
            assert_eq!(version.major, 2);
            assert_eq!(version.minor, 0);
            assert_eq!(version.patch, 0);
            assert_eq!(version.pre_release, Some("alpha.1".to_string()));
            assert_eq!(version.build, Some("build.456".to_string()));
        }

        #[test]
        fn test_version_with_complex_prerelease() {
            let cases = vec![
                ("1.0.0-alpha", "alpha"),
                ("1.0.0-alpha.1", "alpha.1"),
                ("1.0.0-0.3.7", "0.3.7"),
                ("1.0.0-x.7.z.92", "x.7.z.92"),
                ("1.0.0-alpha-beta", "alpha-beta"),
            ];

            for (version_str, expected_prerelease) in cases {
                let version = BlueprintVersion::from_str(version_str).unwrap();
                assert_eq!(
                    version.pre_release,
                    Some(expected_prerelease.to_string()),
                    "Failed for version: {}",
                    version_str
                );
            }
        }

        #[test]
        fn test_zero_versions() {
            let version = BlueprintVersion::from_str("0.0.0").unwrap();
            assert_eq!(version.major, 0);
            assert_eq!(version.minor, 0);
            assert_eq!(version.patch, 0);
        }

        #[test]
        fn test_large_version_numbers() {
            let version = BlueprintVersion::from_str("999.888.777").unwrap();
            assert_eq!(version.major, 999);
            assert_eq!(version.minor, 888);
            assert_eq!(version.patch, 777);
        }

        #[test]
        fn test_invalid_version_formats() {
            let invalid_cases = vec![
                "",
                "1",
                "1.2",
                "1.2.3.4",
                "a.b.c",
                "1.a.3",
                "1.2.c",
                "1..3",
                ".1.2",
                "1.2.",
                "-1.2.3",
                "1.-2.3",
                "1.2.-3",
            ];

            for case in invalid_cases {
                let result = BlueprintVersion::from_str(case);
                assert!(result.is_err(), "Expected error for version: {}", case);
            }
        }

        #[test]
        fn test_version_string_roundtrip() {
            let test_cases = vec![
                "1.0.0",
                "1.2.3",
                "0.0.1",
                "999.888.777",
                "1.0.0-alpha",
                "1.0.0+build",
                "1.0.0-alpha+build",
                "2.1.0-beta.1+exp.sha.123",
            ];

            for case in test_cases {
                let version = BlueprintVersion::from_str(case).unwrap();
                let roundtrip = version.to_string();
                assert_eq!(case, roundtrip, "Roundtrip failed for: {}", case);
            }
        }
    }

    mod version_comparison_tests {
        use super::*;

        #[test]
        fn test_version_equality() {
            let v1 = BlueprintVersion::new(1, 2, 3);
            let v2 = BlueprintVersion::new(1, 2, 3);
            let v3 = BlueprintVersion::new(1, 2, 4);

            assert_eq!(v1, v2);
            assert_ne!(v1, v3);
        }

        #[test]
        fn test_version_ordering() {
            let v1_0_0 = BlueprintVersion::new(1, 0, 0);
            let v1_0_1 = BlueprintVersion::new(1, 0, 1);
            let v1_1_0 = BlueprintVersion::new(1, 1, 0);
            let v2_0_0 = BlueprintVersion::new(2, 0, 0);

            // Patch version ordering
            assert!(v1_0_1 > v1_0_0);
            assert!(v1_0_0 < v1_0_1);

            // Minor version ordering
            assert!(v1_1_0 > v1_0_0);
            assert!(v1_1_0 > v1_0_1);

            // Major version ordering
            assert!(v2_0_0 > v1_1_0);
            assert!(v2_0_0 > v1_0_1);
            assert!(v2_0_0 > v1_0_0);
        }

        #[test]
        fn test_prerelease_ordering() {
            let v1_0_0 = BlueprintVersion::new(1, 0, 0);
            let mut v1_0_0_alpha = BlueprintVersion::new(1, 0, 0);
            v1_0_0_alpha.pre_release = Some("alpha".to_string());

            // Pre-release versions are less than normal versions
            assert!(v1_0_0_alpha < v1_0_0);
            assert!(v1_0_0 > v1_0_0_alpha);
        }

        #[test]
        fn test_change_type_detection() {
            let v1_0_0 = BlueprintVersion::new(1, 0, 0);
            let v1_0_1 = BlueprintVersion::new(1, 0, 1);
            let v1_1_0 = BlueprintVersion::new(1, 1, 0);
            let v2_0_0 = BlueprintVersion::new(2, 0, 0);

            // Breaking changes
            assert!(v2_0_0.is_breaking_change_from(&v1_0_0));
            assert!(!v1_1_0.is_breaking_change_from(&v1_0_0));
            assert!(!v1_0_1.is_breaking_change_from(&v1_0_0));

            // Feature changes
            assert!(v1_1_0.is_feature_change_from(&v1_0_0));
            assert!(!v2_0_0.is_feature_change_from(&v1_0_0));
            assert!(!v1_0_1.is_feature_change_from(&v1_0_0));

            // Patch changes
            assert!(v1_0_1.is_patch_change_from(&v1_0_0));
            assert!(!v1_1_0.is_patch_change_from(&v1_0_0));
            assert!(!v2_0_0.is_patch_change_from(&v1_0_0));
        }

        #[test]
        fn test_backwards_compatibility_detection() {
            let v1_0_0 = BlueprintVersion::new(1, 0, 0);
            let v0_9_0 = BlueprintVersion::new(0, 9, 0);
            let v1_1_0 = BlueprintVersion::new(1, 1, 0);
            let v2_0_0 = BlueprintVersion::new(2, 0, 0);

            // Backwards comparisons should return false
            assert!(!v0_9_0.is_breaking_change_from(&v1_0_0));
            assert!(!v0_9_0.is_feature_change_from(&v1_0_0));
            assert!(!v0_9_0.is_patch_change_from(&v1_0_0));

            // Same version comparisons
            assert!(!v1_0_0.is_breaking_change_from(&v1_0_0));
            assert!(!v1_0_0.is_feature_change_from(&v1_0_0));
            assert!(!v1_0_0.is_patch_change_from(&v1_0_0));
        }
    }

    mod version_increment_tests {
        use super::*;

        #[test]
        fn test_patch_increment() {
            let mut version = BlueprintVersion::new(1, 2, 3);
            version.increment_patch();

            assert_eq!(version.major, 1);
            assert_eq!(version.minor, 2);
            assert_eq!(version.patch, 4);
            assert_eq!(version.pre_release, None);
            assert_eq!(version.build, None);
        }

        #[test]
        fn test_minor_increment() {
            let mut version = BlueprintVersion::new(1, 2, 3);
            version.increment_minor();

            assert_eq!(version.major, 1);
            assert_eq!(version.minor, 3);
            assert_eq!(version.patch, 0);
            assert_eq!(version.pre_release, None);
            assert_eq!(version.build, None);
        }

        #[test]
        fn test_major_increment() {
            let mut version = BlueprintVersion::new(1, 2, 3);
            version.increment_major();

            assert_eq!(version.major, 2);
            assert_eq!(version.minor, 0);
            assert_eq!(version.patch, 0);
            assert_eq!(version.pre_release, None);
            assert_eq!(version.build, None);
        }

        #[test]
        fn test_increment_clears_prerelease_and_build() {
            let mut version = BlueprintVersion::from_str("1.2.3-alpha+build").unwrap();

            version.increment_patch();
            assert_eq!(version.to_string(), "1.2.4");

            let mut version = BlueprintVersion::from_str("1.2.3-beta+build123").unwrap();
            version.increment_minor();
            assert_eq!(version.to_string(), "1.3.0");

            let mut version = BlueprintVersion::from_str("1.2.3-rc1+build456").unwrap();
            version.increment_major();
            assert_eq!(version.to_string(), "2.0.0");
        }

        #[test]
        fn test_increment_overflow_safety() {
            // Test with large numbers near overflow
            let mut version = BlueprintVersion::new(u32::MAX - 1, u32::MAX - 1, u32::MAX - 1);

            version.increment_patch();
            assert_eq!(version.patch, u32::MAX);

            let mut version = BlueprintVersion::new(u32::MAX - 1, u32::MAX - 1, 0);
            version.increment_minor();
            assert_eq!(version.minor, u32::MAX);
            assert_eq!(version.patch, 0);

            let mut version = BlueprintVersion::new(u32::MAX - 1, 0, 0);
            version.increment_major();
            assert_eq!(version.major, u32::MAX);
            assert_eq!(version.minor, 0);
            assert_eq!(version.patch, 0);
        }
    }

    mod version_edge_cases {
        use super::*;

        #[test]
        fn test_display_trait() {
            let version = BlueprintVersion::new(1, 2, 3);
            assert_eq!(format!("{}", version), "1.2.3");

            let version = BlueprintVersion::from_str("1.0.0-alpha+build").unwrap();
            assert_eq!(format!("{}", version), "1.0.0-alpha+build");
        }

        #[test]
        fn test_from_str_trait() {
            use std::str::FromStr;
            
            let version: BlueprintVersion = "1.2.3".parse().unwrap();
            assert_eq!(version.major, 1);
            assert_eq!(version.minor, 2);
            assert_eq!(version.patch, 3);
        }

        #[test]
        fn test_clone_and_debug() {
            let version = BlueprintVersion::from_str("1.2.3-alpha+build").unwrap();
            let cloned = version.clone();
            
            assert_eq!(version, cloned);
            
            // Test debug formatting doesn't panic
            let debug_str = format!("{:?}", version);
            assert!(debug_str.contains("BlueprintVersion"));
        }

        #[test]
        fn test_serialization() {
            let version = BlueprintVersion::from_str("1.2.3-alpha+build").unwrap();
            
            // Test serialization doesn't panic
            let json = serde_json::to_string(&version).unwrap();
            let deserialized: BlueprintVersion = serde_json::from_str(&json).unwrap();
            
            assert_eq!(version, deserialized);
        }

        #[test]
        fn test_version_with_unicode_prerelease() {
            // Test that unicode in prerelease/build is handled
            let version = BlueprintVersion::from_str("1.0.0-αlpha+βuild").unwrap();
            assert_eq!(version.pre_release, Some("αlpha".to_string()));
            assert_eq!(version.build, Some("βuild".to_string()));
            assert_eq!(version.to_string(), "1.0.0-αlpha+βuild");
        }

        #[test]
        fn test_version_with_numbers_in_prerelease() {
            let version = BlueprintVersion::from_str("1.0.0-alpha.1.2.3+build.456.789").unwrap();
            assert_eq!(version.pre_release, Some("alpha.1.2.3".to_string()));
            assert_eq!(version.build, Some("build.456.789".to_string()));
        }
    }

    mod version_performance_tests {
        use super::*;
        use std::time::Duration;
        use crate::blueprint::tests::TestAssertions;

        #[test]
        fn test_version_parsing_performance() {
            let test_versions = vec![
                "1.0.0", "1.2.3", "10.20.30", "999.888.777",
                "1.0.0-alpha", "1.0.0+build", "1.0.0-alpha+build",
                "2.1.0-beta.1+exp.sha.123456789abcdef"
            ];

            let start = std::time::Instant::now();
            for _ in 0..1000 {
                for version_str in &test_versions {
                    BlueprintVersion::from_str(version_str).unwrap();
                }
            }
            let duration = start.elapsed();

            // Parsing should be fast - 1000 iterations of 8 versions should be under 100ms
            TestAssertions::assert_duration_within(
                duration,
                Duration::from_millis(50),
                Duration::from_millis(100)
            );
        }

        #[test]
        fn test_version_comparison_performance() {
            let versions: Vec<BlueprintVersion> = (0..100)
                .map(|i| BlueprintVersion::new(i / 10, i % 10, i % 5))
                .collect();

            let start = std::time::Instant::now();
            for _ in 0..1000 {
                for i in 0..versions.len() - 1 {
                    let _cmp = versions[i] < versions[i + 1];
                }
            }
            let duration = start.elapsed();

            // Comparisons should be very fast
            TestAssertions::assert_duration_within(
                duration,
                Duration::from_millis(10),
                Duration::from_millis(50)
            );
        }
    }
}