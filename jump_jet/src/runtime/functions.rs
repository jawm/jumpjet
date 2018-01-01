use std::error::Error;

use runtime::language_types::ValueTypeInstance;

use tree::functions::Function;
use tree::language_types::ValueType;

impl Function {
    pub fn check_arguments(&self, arguments: &Vec<ValueTypeInstance>) -> Result<(),&Error> {
        if arguments.len() != self.signature.parameters.len() {
            panic!("wrong number of arguments provided");
        }
        for (parameter, argument) in self.signature.parameters.iter().zip(arguments.iter()) {
            match parameter {
                &ValueType::I32 => {if let &ValueTypeInstance::I32(_) = argument{} else {
                    panic!("don't know something errror i32 {:?}", argument);
                }},
                &ValueType::I64 => {if let &ValueTypeInstance::I64(_) = argument{} else {
                    panic!("don't know something errror i64");
                }},
                &ValueType::F32 => {if let &ValueTypeInstance::F32(_) = argument{} else {
                    panic!("don't know something errror f32");
                }},
                &ValueType::F64 => {if let &ValueTypeInstance::F64(_) = argument{} else {
                    panic!("don't know something errror f64");
                }}
            }
        }
        Ok(())
    }

    pub fn execute(&self, arguments: Vec<ValueTypeInstance>) -> Result<Vec<ValueTypeInstance>,&Error> {
        panic!("ayy");
    }
}