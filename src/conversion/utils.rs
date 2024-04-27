use rasn_compiler::intermediate::{
    constraints::Constraint,
    information_object::{InformationObjectClass, InformationObjectField},
    types::{Choice, SequenceOrSet},
    ASN1Type, ASN1Value, CharacterStringType, IntegerType,
};
use rasn_compiler::prelude:: {*, ir::*};

use crate::common::{IntegerTypeExt, to_ros_title_case};

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

pub fn format_comments(comments: &str) -> Result<String, GeneratorError> {
    if comments.is_empty() {
        Ok("".into())
    } else {
        let joined = String::from("// ") + &comments.replace('\n', "\n//") + "\n";
        Ok(joined)
    }
}

#[derive(Clone)]
pub struct NameType {
    pub name: String,
    pub ty: String,
    pub is_primitive: bool,
}


pub fn inner_name(name: &String, parent_name: &String) -> String {
    format!("{}{}", parent_name, name)
}

pub struct NamedSeqMember {
    pub name_type: NameType,
    pub is_optional: bool,
    pub has_default: bool,
}

pub fn get_sequence_or_set_members_names(
    sequence_or_set: &SequenceOrSet,
) -> Vec<NamedSeqMember> {
    sequence_or_set.members
        .iter().
        map(|member| 
            NamedSeqMember {
            name_type: NameType {
                name: member.name.clone(),
                ty: constraints_and_type_name(&member.ty, &member.name, &"".to_string()).unwrap().1,
                is_primitive: !matches!(member.ty, ASN1Type::ElsewhereDeclaredType(_))
            }, 
            is_optional: member.is_optional,
            has_default: member.default_value.is_some()
            }
        )
        .collect::<Vec<NamedSeqMember>>()
}

pub fn get_choice_members_names(
    choice: &Choice,
) -> Vec<NameType> {
    choice.options
        .iter().
        map(|member| (
            NameType {
                name: member.name.clone(),
                ty: constraints_and_type_name(&member.ty, &member.name, &"".to_string()).unwrap().1,
                is_primitive: !matches!(member.ty, ASN1Type::ElsewhereDeclaredType(_))
            }
        ))
        .collect::<Vec<NameType>>()
}

fn constraints_and_type_name(
    ty: &ASN1Type,
    name: &String,
    parent_name: &String,
) -> Result<(Vec<Constraint>, String), GeneratorError> {
    Ok(match ty {
        ASN1Type::Null => (vec![], "byte".into()),
        ASN1Type::Boolean(b) => (b.constraints.clone(), "BOOLEAN".into()),
        ASN1Type::Integer(i) => (i.constraints.clone(), "INTEGER".into()),/*{
            let per_constraints = per_visible_range_constraints(true, &i.constraints)?;
            (
                i.constraints.clone(),
                int_type_token(
                    per_constraints.min(),
                    per_constraints.max(),
                    per_constraints.is_extensible(),
                ),
            )
        }*/
        ASN1Type::Real(_) => (vec![], "float64".into()),
        ASN1Type::ObjectIdentifier(_o) => todo!(),
        ASN1Type::BitString(_b) => todo!(),
        ASN1Type::OctetString(o) => (o.constraints.clone(), "uint8[]".into()),
        ASN1Type::GeneralizedTime(_o) => todo!(),
        ASN1Type::UTCTime(_o) => todo!(),
        ASN1Type::Time(_t) => todo!(),
        ASN1Type::CharacterString(c) => (c.constraints.clone(), string_type(&c.ty).unwrap_or("STRING".into())),
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
        CharacterStringType::IA5String => Ok("IA5String".into()),
        CharacterStringType::TeletexString => Ok("TeletexString".into()),
        CharacterStringType::VideotexString => Ok("VideotexString".into()),
        CharacterStringType::GraphicString => Ok("GraphicString".into()),
        CharacterStringType::GeneralString => Ok("GeneralString".into()),
        CharacterStringType::UniversalString => Ok("UniversalString".into()),
        CharacterStringType::UTF8String => Ok("UTF8String".into()),
        CharacterStringType::BMPString => Ok("BMPString".into()),
        CharacterStringType::PrintableString => Ok("PrintableString".into()),
    }
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
        ASN1Value::Choice { inner_value, .. } => {
            if let Some(_ty_n) = type_name {
                todo!()
            } else {
                Err(error!(
                    Unidentified,
                    "A type name is needed to stringify choice value {:?}", inner_value
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
                    .map(|(_, _, val)| value_to_tokens(val.value(), None))
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
            ASN1Value::Choice { inner_value, .. } => inner_value.is_const_type(),
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

