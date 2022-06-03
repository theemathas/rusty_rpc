/*
Grammar for the interface specification:

// EBNF grammar for the RPC specification format syntax
// x* means x zero or more times
// x | y means x or y
// "abcd" means literal string "abcd"

// root terminal
specification-document := definition *
definition := service-definition | struct-definition

// mirrors rust's struct definition
struct-definition := "struct" identifier "{" struct-field * "}"
struct-field := identifier ":" type ","

service-definition := "service" identifier "{" service-method * "}"
// TODO add &mut self
service-method := identifier "(" ( "&" "self" ) ( "," identifier ":" type )* ")" "->" type ";"

// TODO add "&mut"
return-type := "&" service-type | data-type
data-type := "i32" | struct-type
struct-type := identifier

identifier := A string that starts with an alphanumberic character followed by zero or more alphanumberic characters and/or underscores. Except that it must not match a reserved word.

Reserved word list: "struct", "service", "self", "mut", "crate", "super", "Self".
Note: "crate", "super" and "Self" are otherwise in the grammar, but are reserved because Rust identifiers cannot be these keywords,
even when using raw identifiers. See https://doc.rust-lang.org/1.60.0/reference/identifiers.html
*/

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::{
        complete::{multispace0, multispace1, satisfy},
        is_alphabetic, is_alphanumeric,
    },
    combinator::{eof, map, map_res, value, verify},
    error::ParseError,
    multi::many0,
    sequence::{pair, preceded, terminated, tuple},
    IResult, Parser,
};
use std::{
    collections::{hash_map::Entry, HashMap},
    iter::once,
};

use crate::interface::{DataType, Identifier, Method, ReturnType, RpcInterface, Service, Struct};

pub fn parse_interface(input: &[u8]) -> IResult<&[u8], RpcInterface> {
    enum Definition {
        Struct(Identifier, Struct),
        Service(Identifier, Service),
    }

    // Parser that returns Vec<Definition>
    let parse_definitions = many0_padded_by_multispace(alt((
        map(parse_struct, |(x, y)| Definition::Struct(x, y)),
        map(parse_service, |(x, y)| Definition::Service(x, y)),
    )));

    fn definitions_to_interface(definitions: Vec<Definition>) -> Result<RpcInterface, String> {
        let mut output = RpcInterface {
            structs: HashMap::new(),
            services: HashMap::new(),
        };
        for definition in definitions {
            // Insert the definition in to the appropriate struct in `output`.
            // If there's a duplicate definition name, report an error.
            match definition {
                Definition::Struct(x, y) => {
                    match output.structs.entry(x) {
                        Entry::Vacant(entry) => entry.insert(y),
                        Entry::Occupied(entry) => {
                            let msg = format!("Duplicate struct definition: {:?}", entry.key());
                            eprintln!("{msg}");
                            return Err(msg);
                        }
                    };
                }
                Definition::Service(x, y) => {
                    match output.services.entry(x) {
                        Entry::Vacant(entry) => entry.insert(y),
                        Entry::Occupied(entry) => {
                            let msg = format!("Duplicate service definition: {:?}", entry.key());
                            eprintln!("{msg}");
                            return Err(msg);
                        }
                    };
                }
            };
        }
        Ok(output)
    }

    terminated(map_res(parse_definitions, definitions_to_interface), eof)(input)
}

fn parse_struct(input: &[u8]) -> IResult<&[u8], (Identifier, Struct)> {
    map_res(
        tuple((
            tag("struct"),
            multispace1,
            parse_identifier,
            multispace0,
            tag("{"),
            many0_padded_by_multispace(parse_struct_field),
            tag("}"),
        )),
        |(_, _, struct_name, _, _, field_vec, _)| -> _ {
            let mut field_map = HashMap::<Identifier, DataType>::new();
            for (field_name, field_type) in field_vec {
                match field_map.entry(field_name) {
                    Entry::Vacant(entry) => entry.insert(field_type),
                    Entry::Occupied(entry) => {
                        let msg = format!("Duplicate struct field definition: {:?}", entry.key());
                        eprintln!("{msg}");
                        return Err(msg);
                    }
                };
            }
            Ok((struct_name, Struct { fields: field_map }))
        },
    )(input)
}

fn parse_struct_field(input: &[u8]) -> IResult<&[u8], (Identifier, DataType)> {
    map(
        tuple((
            parse_identifier,
            multispace0,
            tag(":"),
            multispace0,
            parse_data_type,
            multispace0,
            tag(","),
        )),
        |(field_name, _, _, _, field_type, _, _)| (field_name, field_type),
    )(input)
}

fn parse_service(input: &[u8]) -> IResult<&[u8], (Identifier, Service)> {
    map_res(
        tuple((
            tag("service"),
            multispace1,
            parse_identifier,
            multispace0,
            tag("{"),
            many0_padded_by_multispace(parse_method),
            tag("}"),
        )),
        |(_, _, service_name, _, _, method_vec, _)| -> _ {
            let mut method_map = HashMap::<Identifier, Method>::new();
            for (method_name, method_type) in method_vec {
                match method_map.entry(method_name) {
                    Entry::Vacant(entry) => entry.insert(method_type),
                    Entry::Occupied(entry) => {
                        let msg = format!("Duplicate struct method definition: {:?}", entry.key());
                        eprintln!("{msg}");
                        return Err(msg);
                    }
                };
            }
            Ok((
                service_name,
                Service {
                    methods: method_map,
                },
            ))
        },
    )(input)
}

fn parse_method(input: &[u8]) -> IResult<&[u8], (Identifier, Method)> {
    let parse_parameter = map(
        tuple((
            tag(","),
            multispace0,
            parse_identifier,
            multispace0,
            tag(":"),
            multispace0,
            parse_data_type,
        )),
        |(_, _, param_name, _, _, _, param_type)| (param_name, param_type),
    );
    map(
        tuple((
            parse_identifier,
            multispace0,
            tag("("),
            multispace0,
            tag("&"),
            multispace0,
            tag("self"),
            many0_padded_by_multispace(parse_parameter),
            tag(")"),
            multispace0,
            tag("->"),
            multispace0,
            parse_return_type,
            multispace0,
            tag(";"),
        )),
        |(method_name, _, _, _, _, _, _, non_self_params, _, _, _, _, return_type, _, _)| {
            (
                method_name,
                Method {
                    non_self_params,
                    return_type,
                },
            )
        },
    )(input)
}

fn parse_return_type(input: &[u8]) -> IResult<&[u8], ReturnType> {
    let parse_service_type = map(
        tuple((
            tag("&"),
            multispace0,
            tag("service"),
            multispace1,
            parse_identifier,
        )),
        |(_, _, _, _, x)| ReturnType::ServiceRef(x),
    );
    alt((parse_service_type, parse_data_type.map(ReturnType::Data)))(input)
}

fn parse_data_type(input: &[u8]) -> IResult<&[u8], DataType> {
    alt((
        value(DataType::I32, tag("i32")),
        map(parse_identifier, DataType::Struct),
    ))(input)
}

fn parse_identifier(input: &[u8]) -> IResult<&[u8], Identifier> {
    // This parses an identifier except it returns a String and it lets through keywords.
    let parse_almost_identifier = pair(
        satisfy(|ch| is_alphabetic(ch as u8)),
        many0(satisfy(|ch| is_alphanumeric(ch as u8) || ch == '_')),
    )
    .map(|(first, rest)| once(first).chain(rest).collect::<String>());

    map(
        verify(parse_almost_identifier, |s: &String| {
            // I hate this syntax lol
            !["struct", "service", "self", "mut", "crate", "super", "Self"].contains(&&**s)
        }),
        Identifier,
    )(input)
}

// Like many0, but with optional multispace in between, at the beginning, and at the end.
fn many0_padded_by_multispace<'a, O, E, F>(
    parser: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Vec<O>, E>
where
    F: Parser<&'a [u8], O, E>,
    E: ParseError<&'a [u8]>,
{
    preceded(multispace0, many0(terminated(parser, multispace0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_interface() {
        let input = r#"
            struct Foo {
                x : i32 ,
                y : Foo ,
            }

            service MyService {
                foo ( & self ) -> i32 ;
                bar ( & self , arg1 : i32 , arg2 : Foo ) -> Foo ;
                baz ( & self ) -> & service MyService ;
            }
        "#;
        let ident = |s: &str| Identifier(s.to_string());
        let foo_ident = || ident("Foo");
        let expected = RpcInterface {
            structs: HashMap::from([(
                foo_ident(),
                Struct {
                    fields: HashMap::from([
                        (ident("x"), DataType::I32),
                        (ident("y"), DataType::Struct(foo_ident())),
                    ]),
                },
            )]),
            services: HashMap::from([(
                ident("MyService"),
                Service {
                    methods: HashMap::from([
                        (
                            ident("foo"),
                            Method {
                                non_self_params: vec![],
                                return_type: ReturnType::Data(DataType::I32),
                            },
                        ),
                        (
                            ident("bar"),
                            Method {
                                non_self_params: vec![
                                    (ident("arg1"), DataType::I32),
                                    (ident("arg2"), DataType::Struct(foo_ident())),
                                ],
                                return_type: ReturnType::Data(DataType::Struct(foo_ident())),
                            },
                        ),
                        (
                            ident("baz"),
                            Method {
                                non_self_params: vec![],
                                return_type: ReturnType::ServiceRef(ident("MyService")),
                            },
                        ),
                    ]),
                },
            )]),
        };
        assert_eq!(
            Ok((&[] as &[u8], expected)),
            parse_interface(input.as_bytes())
        );
    }
}
