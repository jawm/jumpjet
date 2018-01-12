use std::io::Read;

use parser::leb::ReadLEB;
use parser::ParseError;

use parse_tree::globals::Global;
use parse_tree::language_types::GlobalType;
use parse_tree::language_types::InitExpression;
use parse_tree::language_types::ValueType;
use parse_tree::ParseModule;

pub fn parse(reader: &mut Read, module: &mut ParseModule) -> Result<(), ParseError> {
    debug!("Parsing globals section");
    let count = reader.bytes().read_varuint(32).unwrap();
    for _ in 0..count {
        let constraints = GlobalType::parse(reader)?;
        let init_expr = InitExpression::parse(reader, module);
        match init_expr {
            Ok(InitExpression::I32Const(_)) => {
                if constraints.content_type != ValueType::I32 {
                    return Err(ParseError::CustomError("Global initialiser type doesn't match it's type".to_string()));
                }
            },
            Ok(InitExpression::I64Const(_)) => {
                if constraints.content_type != ValueType::I64 {
                    return Err(ParseError::CustomError("Global initialiser type doesn't match it's type".to_string()));
                }
            },
            Ok(InitExpression::F32Const(_)) => {
                if constraints.content_type != ValueType::F32 {
                    return Err(ParseError::CustomError("Global initialiser type doesn't match it's type".to_string()));
                }
            },
            Ok(InitExpression::F64Const(_)) => {
                if constraints.content_type != ValueType::F64 {
                    return Err(ParseError::CustomError("Global initialiser type doesn't match it's type".to_string()));
                }
            },
            Ok(_) => {return Err(ParseError::CustomError("Global initialiser type must be for value type".to_string()))},
            Err(e) => {return Err(e);}
        }
        if let Ok(value) = init_expr {
            module.globals.push(Global{
                constraints,
                value
            });
        }
    }
    Ok(())
}