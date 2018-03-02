use std::io::Read;

use std::collections::HashMap;

use parser::ModuleParser;
use parser::ParseError;

use runtime_tree::ExternalKindInstance;
use runtime_tree::ModuleTemplate;
use runtime_tree::ModuleTemplateBuilder;

#[macro_use]
pub mod exports;
pub mod language_types;
pub mod functions;

pub fn instantiate(reader: &mut Read, imports: HashMap<String, HashMap<String, ExternalKindInstance>>) -> Result<ModuleTemplate, ParseError> {
    info!("Attempting to parse WebAssembly module");
    let parser = ModuleParser::default();
    parser.parse_module(reader).unwrap().build(imports)
}

/*
// experimental ideas about future API:

macro_rules! wasm {
    ($module_name:ident {
        $(fn $fn_name:ident(
            $($arg_name:ident:$arg_type:ty),*
        )->$return_type:ty;)*
    }) => {
        struct $module_name {

        }

        impl $module_name {
            pub fn from(_: &str) -> Self {
                $module_name {}
            }

            pub fn instantiate(&self) -> &Self {
                self
            }

            $(
            pub fn $fn_name(&self, $($arg_name: $arg_type),*) {
                let mut args = vec![];
                $(args.push($arg_name.to_vtp());)*
                println!("{:?}", args);
            }
            )*
        }
    };

    (@fn) => {}
}

#[derive(Debug)]
enum VTP {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64)
}

trait ToVTP {
    fn to_vtp(self) -> VTP;
}

impl ToVTP for i32 {
    fn to_vtp(self) -> VTP {
        VTP::I32(self)
    }
}

impl ToVTP for i64 {
    fn to_vtp(self) -> VTP {
        VTP::I64(self)
    }
}

impl ToVTP for f32 {
    fn to_vtp(self) -> VTP {
        VTP::F32(self)
    }
}

impl ToVTP for f64 {
    fn to_vtp(self) -> VTP {
        VTP::F64(self)
    }
}

fn main() {

    wasm!(ModuleName {
        fn add(a:i32, b:i32, c: f32)->i32;
        fn sub(a:i64, b:i64)->i64;
    });

    let module_template = ModuleName::from("hi");
    module_template.instantiate().add(10, 5, 3.14);
}

*/