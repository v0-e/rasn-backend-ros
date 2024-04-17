#[macro_export]
macro_rules! e2e_pdu {
    ($suite:ident, $asn1:literal, $expected:literal) => {
        #[test]
        fn $suite() {
            assert_eq!(
                rasn_compiler::Compiler::new()
                    .with_backend(ros_backend::ros::Ros)
                    .add_asn_literal(&format!(
                        "TestModule DEFINITIONS AUTOMATIC TAGS::= BEGIN {} END",
                        $asn1
                    ))
                    .compile_to_string()
                    .unwrap()
                    .generated
                    .replace("#<typedef>\n", "")
                    .replace("\n#</typedef>", "")
                    .lines()
                    .skip(1)
                    .collect::<Vec<&str>>()
                    .join("\n")
                    .replace(|c: char| c.is_whitespace(), ""),
                format!("{}", $expected)
                    .to_string()
                    .replace(|c: char| c.is_whitespace(), ""),
            )
        }
    };
}
