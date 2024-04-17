use rasn_compiler::intermediate::{
    constraints::Constraint,
    encoding_rules::per_visible::{
        per_visible_range_constraints, CharsetSubset, PerVisibleAlphabetConstraints,
    },
    information_object::{InformationObjectClass, InformationObjectField},
    types::{Choice, ChoiceOption, Enumerated, SequenceOrSet, SequenceOrSetMember},
    ASN1Type, ASN1Value, CharacterStringType, IntegerType,
    ToplevelDefinition, ToplevelTypeDefinition,
};

use rasn_compiler::prelude::ir::*;
use rasn_compiler::prelude::*;

macro_rules! error {
    ($kind:ident, $($arg:tt)*) => {
        GeneratorError {
            details: format!($($arg)*),
            top_level_declaration: None,
            kind: GeneratorErrorType::$kind,
        }
    };
}

pub(crate) use error;

use super::*;

pub trait IntegerTypeExt {
    fn to_str(self) -> &'static str;
}

impl IntegerTypeExt for IntegerType {
    fn to_str(self) -> &'static str {
        match self {
            IntegerType::Int8 => "int8",
            IntegerType::Uint8 => "uint8",
            IntegerType::Int16 => "int16",
            IntegerType::Uint16 => "uint16",
            IntegerType::Int32 => "int32",
            IntegerType::Uint32 => "uint32",
            IntegerType::Int64 => "int64",
            IntegerType::Uint64 => "uint64",
            IntegerType::Unbounded => "int64",
        }
    }
}

pub fn to_ros_snake_case(input: &str) -> String {
    let input = input.replace('-', "_");
    let mut lowercase = String::with_capacity(input.len());

    let peekable = &mut input.chars().peekable();
    while let Some(c) = peekable.next() {
        if c.is_lowercase() || c.is_numeric() {
            lowercase.push(c);
            if c != '_' && peekable.peek().map_or(false, |next| next.is_uppercase()) {
                lowercase.push('_');
            }
        } else {
            lowercase.push(c.to_ascii_lowercase());
        }
    }
    lowercase
}

pub fn to_ros_const_case(input: &str) -> String {
    to_ros_snake_case(input).to_string().to_uppercase()
}

pub fn to_ros_title_case(input: &str) -> String {
    input.replace('-', "")
}

pub fn format_comments(comments: &str) -> Result<String, GeneratorError> {
    if comments.is_empty() {
        Ok("".into())
    } else {
        let joined = String::from("# ") + &comments.replace('\n', "\n#") + "\n";
        Ok(joined)
    }
}

pub fn inner_name(name: &String, parent_name: &String) -> String {
    format!("{}{}", parent_name, name)
}

pub fn int_type_token(opt_min: Option<i128>, opt_max: Option<i128>, is_extensible: bool) -> String {
    if let (Some(min), Some(max)) = (opt_min, opt_max) {
        format!(
            "{}",
            if is_extensible {
                "int64"
            } else if min >= 0 {
                match max {
                    r if r <= u8::MAX.into() => "uint8",
                    r if r <= u16::MAX.into() => "uint16",
                    r if r <= u32::MAX.into() => "uint32",
                    r if r <= u64::MAX.into() => "uint64",
                    _ => "uint64",
                }
            } else {
                match (min, max) {
                    (mi, ma) if mi >= i8::MIN.into() && ma <= i8::MAX.into() => "int8",
                    (mi, ma) if mi >= i16::MIN.into() && ma <= i16::MAX.into() => "int16",
                    (mi, ma) if mi >= i32::MIN.into() && ma <= i32::MAX.into() => "int32",
                    (mi, ma) if mi >= i64::MIN.into() && ma <= i64::MAX.into() => "int64",
                    _ => "int64",
                }
            }
        )
    } else {
        format!("int64")
    }
}

pub fn format_constraints(
    signed: bool,
    constraints: &Vec<Constraint>,
) -> Result<String, GeneratorError> {
    if constraints.is_empty() {
        return Ok("".into());
    }
    let per_constraints = per_visible_range_constraints(signed, constraints)?;
    let range_type = int_type_token(
        per_constraints.min::<i128>(),
        per_constraints.max::<i128>(),
        per_constraints.is_extensible(),
    );
    let range_prefix = if per_constraints.is_size_constraint() {
        "LENGTH"
    } else {
        "VALUE"
    };
    // handle default size constraints
    if per_constraints.is_size_constraint()
        && !per_constraints.is_extensible()
        && per_constraints.min::<i128>() == Some(0)
        && per_constraints.max::<i128>().is_none()
    {
        return Ok("".into());
    }
    Ok(
        match (
            per_constraints.min::<i128>(),
            per_constraints.max::<i128>(),
            per_constraints.is_extensible(),
        ) {
            (Some(min), Some(max), true) if min == max => {
                format!(
                    "{range_type} {range_prefix}_MIN = {min}\n\
                     {range_type} {range_prefix}_MAX = {max}"
                )
            }
            (Some(min), Some(max), true) => {
                format!(
                    "{range_type} {range_prefix}_MIN = {min}\n\
                     {range_type} {range_prefix}_MAX = {max}"
                )
            }
            (Some(min), Some(max), false) if min == max => {
                format!(
                    "{range_type} {range_prefix} = {min}"
                )
            }
            (Some(min), Some(max), false) => {
                format!(
                    "{range_type} {range_prefix}_MIN = {min}\n\
                     {range_type} {range_prefix}_MAX = {max}"
                )
            }
            (Some(min), None, true) => {
                format!("{range_type} {range_prefix}_MIN = {min}")
            }
            (Some(min), None, false) => {
                format!("{range_type} {range_prefix}_MIN = {min}")
            }
            (None, Some(max), true) => {
                format!("{range_type} {range_prefix}_MAX = {max}")
            }
            (None, Some(max), false) => {
                format!("{range_type} {range_prefix}_MAX = {max}")
            }
            _ => "".into(),
        },
    )
}

pub fn format_distinguished_values(dvalues: &Option<Vec<DistinguishedValue>>) -> String {
    let mut result = String::from("");
    if let Some(dvalues) = dvalues {
        dvalues.iter().for_each(|dvalue| {
            result.push_str(&format!(
                "{{type}} {{prefix}}{} = {}\n", to_ros_const_case(&dvalue.name), dvalue.value
            ));
        });
    }
    result
}

pub fn _format_alphabet_annotations(
    string_type: CharacterStringType,
    constraints: &Vec<Constraint>,
) -> Result<String, GeneratorError> {
    if constraints.is_empty() {
        return Ok("".into());
    }
    let mut permitted_alphabet = PerVisibleAlphabetConstraints::default_for(string_type);
    for c in constraints {
        if let Some(mut p) = PerVisibleAlphabetConstraints::try_new(c, string_type)? {
            permitted_alphabet += &mut p
        }
    }
    permitted_alphabet.finalize();
    let alphabet_unicode = permitted_alphabet
        .charset_subsets()
        .iter()
        .map(|subset| match subset {
            CharsetSubset::Single(c) => format!("{}", c.escape_unicode()),
            CharsetSubset::Range { from, to } => format!(
                "{}..{}",
                from.map_or(String::from(""), |c| format!("{}", c.escape_unicode())),
                to.map_or(String::from(""), |c| format!("{}", c.escape_unicode()))
            ),
        })
        .collect::<Vec<String>>()
        .join(", ");
    Ok(if alphabet_unicode.is_empty() {
        "".into()
    } else {
        "from(#alphabet_unicode)".into()
    })
}

pub fn format_enum_members(enumerated: &Enumerated) -> String {
    let first_extension_index = enumerated.extensible;
    enumerated
        .members
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let name = to_ros_const_case(&e.name);
            let index = e.index;
            let extension = if i >= first_extension_index.unwrap_or(usize::MAX) {
                "# .extended\n".to_string()
            } else {
                "".to_string()
            };
            String::from(&format!("{extension}uint8 {name} = {index}"))
        })
        .fold("".to_string(), |mut acc, e| {
            acc.push_str(&e);
            acc.push_str("\n");
            acc
        })
}

pub fn format_sequence_or_set_members(
    sequence_or_set: &SequenceOrSet,
    parent_name: &String,
) -> Result<String, GeneratorError> {
    let first_extension_index = sequence_or_set.extensible;
    sequence_or_set.members.iter().enumerate().try_fold(
        "".to_string(),
        |mut acc, (i, m)| {
            let extension_annotation = if i >= first_extension_index.unwrap_or(usize::MAX)
                && m.name.starts_with("ext_group_")
            {
                "extension_addition_group".into()
            } else if i >= first_extension_index.unwrap_or(usize::MAX) {
                "quote!(extension_addition)".into()
            } else {
                "".into()
            };
            format_sequence_member(m, parent_name, extension_annotation).map(
                |declaration| {
                    acc.push_str(&format!("{declaration}"));
                    acc
                },
            )
        },
    )
}

fn format_sequence_member(
    member: &SequenceOrSetMember,
    parent_name: &String,
    _extension_annotation: String,
) -> Result<String, GeneratorError> {
    let name = &member.name;
    let (mut all_constraints, mut formatted_type_name) =
        constraints_and_type_name(&member.ty, &member.name, parent_name)?;
    all_constraints.append(&mut member.constraints.clone());
    let name = to_ros_snake_case(name);
    if (member.is_optional && member.default_value.is_none())
        || member.name.starts_with("ext_group_")
    {
        formatted_type_name = format!("bool {name}_present\n\
                                      {formatted_type_name}")
    }
    Ok(format!("{formatted_type_name} {name}\n"))
}

pub fn format_choice_options(
    choice: &Choice,
    parent_name: &String,
) -> Result<String, GeneratorError> {
    let first_extension_index = choice.extensible;
    let options = choice
        .options
        .iter()
        .enumerate()
        .map(|(i, o)| {
            let extension_annotation = if i >= first_extension_index.unwrap_or(usize::MAX)
                && o.name.starts_with("ext_group_")
            {
                "quote!(extension_addition_group)".into()
            } else if i >= first_extension_index.unwrap_or(usize::MAX) {
                "quote!(extension_addition)".into()
            } else {
                "".into()
            };
            let name = o.name.clone();
            format_choice_option(name, o, parent_name, i, extension_annotation)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let folded_options = options.iter().fold(
        ("".to_string(), "".to_string()),
        |mut acc, (declaration, valset)| {
            acc.0.push_str(&format!("{declaration}\n"));
            acc.1.push_str(&format!("{valset}\n"));
            acc
        },
    );
    Ok(format!("{}\n{}", folded_options.0, folded_options.1))
}

fn format_choice_option(
    name: String,
    member: &ChoiceOption,
    parent_name: &String,
    index: usize,
    _extension_annotation: String,
) -> Result<(String, String), GeneratorError> {
    let (_, formatted_type_name) =
        constraints_and_type_name(&member.ty, &member.name, parent_name)?;
    let choice_type = format!("{formatted_type_name} {}", to_ros_snake_case(&name));
    let choice_selector = format!("uint8 CHOICE_{} = {index}", to_ros_const_case(&name));
    Ok((choice_type, choice_selector))
}

fn constraints_and_type_name(
    ty: &ASN1Type,
    name: &String,
    parent_name: &String,
) -> Result<(Vec<Constraint>, String), GeneratorError> {
    Ok(match ty {
        ASN1Type::Null => (vec![], "byte".into()),
        ASN1Type::Boolean(b) => (b.constraints.clone(), "bool".into()),
        ASN1Type::Integer(i) => {
            let per_constraints = per_visible_range_constraints(true, &i.constraints)?;
            (
                i.constraints.clone(),
                int_type_token(
                    per_constraints.min(),
                    per_constraints.max(),
                    per_constraints.is_extensible(),
                ),
            )
        }
        ASN1Type::Real(_) => (vec![], "float64".into()),
        ASN1Type::ObjectIdentifier(_o) => todo!(),
        ASN1Type::BitString(_b) => todo!(),
        ASN1Type::OctetString(o) => (o.constraints.clone(), "uint8[]".into()),
        ASN1Type::GeneralizedTime(_o) => todo!(),
        ASN1Type::UTCTime(_o) => todo!(),
        ASN1Type::Time(_t) => todo!(),
        ASN1Type::CharacterString(c) => (c.constraints.clone(), "string".into()),
        ASN1Type::Enumerated(_)
        | ASN1Type::Choice(_)
        | ASN1Type::Sequence(_)
        | ASN1Type::SetOf(_)
        | ASN1Type::Set(_) => (vec![], inner_name(name, parent_name)),
        ASN1Type::SequenceOf(s) => {
            let (_, inner_type) = constraints_and_type_name(&s.element_type, name, parent_name)?;
            (s.constraints().clone(), format!("{inner_type}[]").into())
        }
        ASN1Type::ElsewhereDeclaredType(e) => (e.constraints.clone(), to_ros_title_case(&e.identifier)),
        ASN1Type::InformationObjectFieldReference(_)
        | ASN1Type::EmbeddedPdv
        | ASN1Type::External => {
            let tx = &ty.constraints().unwrap()[0];
            let rname = if let Constraint::TableConstraint(ref tc) = tx {
                let v = &tc.object_set.values[0];
                if let ObjectSetValue::Reference(ref r) = v {
                    r.clone()
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            };
            (vec![], rname)
        }
        ASN1Type::ChoiceSelectionType(_) => unreachable!(),
    })
}

pub fn string_type(c_type: &CharacterStringType) -> Result<String, GeneratorError> {
    match c_type {
        CharacterStringType::NumericString => Ok("NumericString".into()),
        CharacterStringType::VisibleString => Ok("VisibleString".into()),
        CharacterStringType::IA5String => Ok("Ia5String".into()),
        CharacterStringType::TeletexString => Ok("TeletexString".into()),
        CharacterStringType::VideotexString => Ok("VideotexString".into()),
        CharacterStringType::GraphicString => Ok("GraphicString".into()),
        CharacterStringType::GeneralString => Ok("GeneralString".into()),
        CharacterStringType::UniversalString => Ok("UniversalString".into()),
        CharacterStringType::UTF8String => Ok("Utf8String".into()),
        CharacterStringType::BMPString => Ok("BmpString".into()),
        CharacterStringType::PrintableString => Ok("PrintableString".into()),
    }
}

pub fn format_default_methods(
    members: &Vec<SequenceOrSetMember>,
    _parent_name: &str,
) -> Result<String, GeneratorError> {
    let mut output = "".to_string();
    for member in members {
        if let Some(value) = member.default_value.as_ref() {
            let val = to_ros_const_case(&value_to_tokens(value, Some(&type_to_tokens(&member.ty)?.to_string()))?);
            // TODO generalize
            let ty = match value {
                ASN1Value::LinkedNestedValue { supertypes: _, value } => {
                    match value.as_ref() {
                        ASN1Value::LinkedIntValue { integer_type, value: _ } => integer_type.to_str().to_string(),
                        _ => type_to_tokens(&member.ty)?,
                    }
                },
                ASN1Value::EnumeratedValue { .. } => {
                    "uint8".into()
                }
                _ => type_to_tokens(&member.ty)?,
            };
            let method_name = format!(
                "{}_DEFAULT",
                to_ros_const_case(&member.name)
            );
            output.push_str(&format!("{ty} {method_name} = {val}\n"));
        }
    }
    Ok(output)
}

pub fn type_to_tokens(ty: &ASN1Type) -> Result<String, GeneratorError> {
    match ty {
        ASN1Type::Null => todo!(),
        ASN1Type::Boolean(_) => Ok("bool".into()),
        ASN1Type::Integer(i) => Ok(i.int_type().to_str().to_string()),
        ASN1Type::Real(_) => Ok("float64".into()),
        ASN1Type::BitString(_) => Ok("BitString".into()),
        ASN1Type::OctetString(_) => Ok("OctetString".into()),
        ASN1Type::CharacterString(CharacterString { ty, .. }) => string_type(ty),
        ASN1Type::Enumerated(_) => Err(error!(
            NotYetInplemented,
            "Enumerated values are currently unsupported!"
        )),
        ASN1Type::Choice(_) => Err(error!(
            NotYetInplemented,
            "Choice values are currently unsupported!"
        )),
        ASN1Type::Sequence(_) => Err(error!(
            NotYetInplemented,
            "Sequence values are currently unsupported!"
        )),
        ASN1Type::SetOf(so) | ASN1Type::SequenceOf(so) => {
            let _inner = type_to_tokens(&so.element_type)?;
            Ok("SequenceOf".into())
        }
        ASN1Type::ObjectIdentifier(_) => Err(error!(
            NotYetInplemented,
            "Object Identifier values are currently unsupported!"
        )),
        ASN1Type::Set(_) => Err(error!(
            NotYetInplemented,
            "Set values are currently unsupported!"
        )),
        ASN1Type::ElsewhereDeclaredType(e) => Ok(e.identifier.clone()),
        ASN1Type::InformationObjectFieldReference(_) => Err(error!(
            NotYetInplemented,
            "Information Object field reference values are currently unsupported!"
        )),
        ASN1Type::Time(_) => Err(error!(
            NotYetInplemented,
            "Time values are currently unsupported!"
        )),
        ASN1Type::GeneralizedTime(_) => Ok("GeneralizedTime".into()),
        ASN1Type::UTCTime(_) => Ok("UtcTime".into()),
        ASN1Type::EmbeddedPdv | ASN1Type::External => Ok("Any".into()),
        ASN1Type::ChoiceSelectionType(c) => {
            let _choice = &c.choice_name;
            let _option = &c.selected_option;
            todo!()
        }
    }
}

pub fn value_to_tokens(
    value: &ASN1Value,
    type_name: Option<&String>,
) -> Result<String, GeneratorError> {
    match value {
        ASN1Value::All => Err(error!(
            NotYetInplemented,
            "All values are currently unsupported!"
        )),
        ASN1Value::Null => todo!(),
        ASN1Value::Choice(i, v) => {
            if let Some(_ty_n) = type_name {
                let _option = i;
                let _inner = value_to_tokens(v, None)?;
                todo!()
            } else {
                Err(error!(
                    Unidentified,
                    "A type name is needed to stringify choice value {:?}", value
                ))
            }
        }
        ASN1Value::OctetString(o) => {
            let _bytes = o.iter().map(|byte| *byte);
            todo!()
        }
        ASN1Value::SequenceOrSet(_) => Err(error!(
            Unidentified,
            "Unexpectedly encountered unlinked struct-like ASN1 value!"
        )),
        ASN1Value::LinkedStructLikeValue(fields) => {
            if let Some(_ty_n) = type_name {
                let _tokenized_fields = fields
                    .iter()
                    .map(|(_, val)| value_to_tokens(val.value(), None))
                    .collect::<Result<Vec<String>, _>>()?;
                todo!()
            } else {
                Err(error!(
                    Unidentified,
                    "A type name is needed to stringify sequence value {:?}", value
                ))
            }
        }
        ASN1Value::Boolean(b) => Ok(b.to_string()),
        ASN1Value::Integer(i) => Ok(i.to_string()),
        ASN1Value::String(s) => Ok(s.to_string()),
        ASN1Value::Real(r) => Ok(r.to_string()),
        ASN1Value::BitString(b) => {
            let _bits = b.iter().map(|bit| bit.to_string());
            todo!()
        }
        ASN1Value::EnumeratedValue {
            enumerated,
            enumerable,
        } => {
            Ok(format!("{}_{}", enumerated, enumerable))
        }
        ASN1Value::LinkedElsewhereDefinedValue { identifier: e, .. }
        | ASN1Value::ElsewhereDeclaredValue { identifier: e, .. } => Ok(e.to_string()),
        ASN1Value::ObjectIdentifier(oid) => {
            let _arcs = oid
                .0
                .iter()
                .filter_map(|arc| arc.number.map(|id| id.to_string()));
            todo!()
        }
        ASN1Value::Time(_t) => match type_name {
            Some(_time_type) => todo!(),
            None => todo!(),
        },
        ASN1Value::LinkedArrayLikeValue(seq) => {
            let _elems = seq
                .iter()
                .map(|v| value_to_tokens(v, None))
                .collect::<Result<Vec<_>, _>>()?;
            todo!()
        }
        ASN1Value::LinkedNestedValue { supertypes: _, value } => {
            Ok(value_to_tokens(value, type_name)?)
        }
        ASN1Value::LinkedIntValue {
            integer_type,
            value,
        } => {
            let val = *value;
            match integer_type {
                IntegerType::Unbounded => Ok(val.to_string()),
                _ => Ok(val.to_string()),
            }
        }
        ASN1Value::LinkedCharStringValue(string_type, value) => {
            let _val = value;
            match string_type {
                CharacterStringType::NumericString => {
                    todo!()
                }
                CharacterStringType::VisibleString => {
                    todo!()
                }
                CharacterStringType::IA5String => {
                    todo!()
                }
                CharacterStringType::UTF8String => todo!(),
                CharacterStringType::BMPString => {
                    todo!()
                }
                CharacterStringType::PrintableString => {
                    todo!()
                }
                CharacterStringType::GeneralString => {
                    todo!()
                }
                CharacterStringType::VideotexString
                | CharacterStringType::GraphicString
                | CharacterStringType::UniversalString
                | CharacterStringType::TeletexString => Err(GeneratorError::new(
                    None,
                    &format!("{:?} values are currently unsupported!", string_type),
                    GeneratorErrorType::NotYetInplemented,
                )),
            }
        }
    }
}

pub fn format_nested_sequence_members(
    sequence_or_set: &SequenceOrSet,
    parent_name: &String,
) -> Result<Vec<String>, GeneratorError> {
    sequence_or_set
        .members
        .iter()
        .filter(|m| needs_unnesting(&m.ty))
        .map(|m| {
            generate(ToplevelDefinition::Type(ToplevelTypeDefinition {
                parameterization: None,
                comments: " Inner type ".into(),
                name: inner_name(&m.name, parent_name).to_string(),
                ty: m.ty.clone(),
                tag: None,
                index: None,
            }))
        })
        .collect::<Result<Vec<_>, _>>()
}

fn needs_unnesting(ty: &ASN1Type) -> bool {
    match ty {
        ASN1Type::Enumerated(_)
        | ASN1Type::Choice(_)
        | ASN1Type::Sequence(_)
        | ASN1Type::Set(_) => true,
        ASN1Type::SequenceOf(SequenceOrSetOf { element_type, .. })
        | ASN1Type::SetOf(SequenceOrSetOf { element_type, .. }) => needs_unnesting(element_type),
        _ => false,
    }
}

pub fn format_nested_choice_options(
    choice: &Choice,
    parent_name: &String,
) -> Result<Vec<String>, GeneratorError> {
    choice
        .options
        .iter()
        .filter(|m| {
            matches!(
                m.ty,
                ASN1Type::Enumerated(_)
                    | ASN1Type::Choice(_)
                    | ASN1Type::Sequence(_)
                    | ASN1Type::SequenceOf(_)
                    | ASN1Type::Set(_)
            )
        })
        .map(|m| {
            generate(ToplevelDefinition::Type(ToplevelTypeDefinition {
                parameterization: None,
                comments: " Inner type ".into(),
                name: inner_name(&m.name, parent_name).to_string(),
                ty: m.ty.clone(),
                tag: None,
                index: None,
            }))
        })
        .collect::<Result<Vec<_>, _>>()
}

pub fn format_sequence_or_set_of_item_type(
    type_name: String,
    first_item: Option<&ASN1Value>,
) -> String {
    match type_name {
        name if name == NULL => todo!(),
        name if name == BOOLEAN => "bool".into(),
        name if name == INTEGER => {
            match first_item {
                Some(ASN1Value::LinkedIntValue { integer_type, .. }) => {
                    integer_type.to_str().into()
                }
                _ => "int64".into(), // best effort
            }
        }
        name if name == BIT_STRING => "BitString".into(),
        name if name == OCTET_STRING => "OctetString".into(),
        name if name == GENERALIZED_TIME => "GeneralizedTime".into(),
        name if name == UTC_TIME => "UtcTime".into(),
        name if name == OBJECT_IDENTIFIER => "ObjectIdentifier".into(),
        name if name == NUMERIC_STRING => "NumericString".into(),
        name if name == VISIBLE_STRING => "VisibleString".into(),
        name if name == IA5_STRING => "IA5String".into(),
        name if name == UTF8_STRING => "UTF8String".into(),
        name if name == BMP_STRING => "BMPString".into(),
        name if name == PRINTABLE_STRING => "PrintableString".into(),
        name if name == GENERAL_STRING => "GeneralString".into(),
        name => name,
    }
}

/// Resolves the custom syntax declared in an information object class' WITH SYNTAX clause
pub fn resolve_standard_syntax(
    class: &InformationObjectClass,
    application: &[InformationObjectField],
) -> Result<(ASN1Value, Vec<(usize, ASN1Type)>), GeneratorError> {
    let mut key = None;
    let mut field_index_map = Vec::<(usize, ASN1Type)>::new();

    let key_index = class
        .fields
        .iter()
        .enumerate()
        .find_map(|(i, f)| f.is_unique.then_some(i))
        .ok_or_else(|| GeneratorError {
            details: format!("Could not find key for class {class:?}"),
            kind: GeneratorErrorType::MissingClassKey,
            ..Default::default()
        })?;

    let mut appl_iter = application.iter().enumerate();
    'syntax_matching: for class_field in &class.fields {
        if let Some((index, field)) = appl_iter.next() {
            if class_field.identifier.identifier() == field.identifier() {
                match field {
                    InformationObjectField::TypeField(f) => {
                        field_index_map.push((index, f.ty.clone()));
                    }
                    InformationObjectField::FixedValueField(f) => {
                        if index == key_index {
                            key = Some(f.value.clone());
                        }
                    }
                    InformationObjectField::ObjectSetField(_) => todo!(),
                }
            } else if !class_field.is_optional {
                return Err(GeneratorError {
                    top_level_declaration: None,
                    details: "Syntax mismatch while resolving information object.".to_string(),
                    kind: GeneratorErrorType::SyntaxMismatch,
                });
            } else {
                continue 'syntax_matching;
            }
        }
    }
    field_index_map.sort_by(|&(a, _), &(b, _)| a.cmp(&b));
    let types = field_index_map.into_iter().collect();
    match key {
        Some(k) => Ok((k, types)),
        None => Err(GeneratorError {
            top_level_declaration: None,
            details: "Could not find class key!".into(),
            kind: GeneratorErrorType::MissingClassKey,
        }),
    }
}

trait ASN1ValueExt {
    fn is_const_type(&self) -> bool;
}

impl ASN1ValueExt for ASN1Value {
    fn is_const_type(&self) -> bool {
        match self {
            ASN1Value::Null | ASN1Value::Boolean(_) | ASN1Value::EnumeratedValue { .. } => true,
            ASN1Value::Choice(_, v) => v.is_const_type(),
            ASN1Value::LinkedIntValue { integer_type, .. } => {
                integer_type != &IntegerType::Unbounded
            }
            ASN1Value::LinkedNestedValue { value, .. } => value.is_const_type(),
            ASN1Value::LinkedElsewhereDefinedValue { can_be_const, .. } => *can_be_const,
            _ => false,
        }
    }
}

impl ASN1ValueExt for ASN1Type {
    fn is_const_type(&self) -> bool {
        match self {
            ASN1Type::Null | ASN1Type::Enumerated(_) | ASN1Type::Boolean(_) => true,
            ASN1Type::Integer(i) => {
                i.constraints.iter().fold(IntegerType::Unbounded, |acc, c| {
                    acc.max_restrictive(c.integer_constraints())
                }) != IntegerType::Unbounded
            }
            ASN1Type::Choice(c) => c
                .options
                .iter()
                .fold(true, |acc, opt| opt.ty.is_const_type() && acc),
            ASN1Type::Set(s) | ASN1Type::Sequence(s) => s
                .members
                .iter()
                .fold(true, |acc, m| m.ty.is_const_type() && acc),
            ASN1Type::SetOf(s) | ASN1Type::SequenceOf(s) => s.element_type.is_const_type(),
            _ => false,
        }
    }
}

