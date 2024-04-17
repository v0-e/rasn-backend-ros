mod utils;

e2e_pdu!(
    single_byte,
    r#" SingleByte ::= INTEGER (0..255)"#,
    r#" uint8 value
        uint8 VALUE_MIN = 0
        uint8 VALUE_MAX = 255 "#
);

e2e_pdu!(
    integer_unconstrained,
    r#" Unbound ::= INTEGER "#,
    r#" int64 value "#
);

e2e_pdu!(
    integer_constrained_positive,
    r#" PositiveNumber ::= INTEGER (0..MAX) "#,
    r#" int64 value 
        int64 VALUE_MIN = 0 "#
);

e2e_pdu!(
    integer_constrained_negative,
    r#" NegativeNumber ::= INTEGER (MIN..-1) "#,
    r#" int64 value 
        int64 VALUE_MAX = -1 "#
);

e2e_pdu!(
    sequence,
    r#" Seq ::= SEQUENCE { aBigNumber INTEGER, anotherBigNumber INTEGER} "#,
    r#" int64 a_big_number 
        int64 another_big_number "#
);

e2e_pdu!(boolean, r#" Maybe ::= BOOLEAN "#, r#" bool value "#);

e2e_pdu!(
    choice,
    r#" Choose ::= CHOICE {aNumber INTEGER, aByteString OCTET STRING}"#,
    r#" uint8 choice

        int64 a_number 
        uint8[] a_byte_string 

        uint8 CHOICE_A_NUMBER = 0
        uint8 CHOICE_A_BYTE_STRING = 1"#
);
