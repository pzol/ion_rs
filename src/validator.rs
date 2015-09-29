use { Ion, Value, Section };
use std::{ fmt, error };

pub struct Validator<'a> {
    ion: &'a Ion,
    spec: &'a Ion,
    errors: Vec<ValidationError>
}


fn dict_table(&(_ , spec) : &(&String, &Value)) -> bool {
    match spec.get("column") {
        Some(&Value::Boolean(b)) => b,
        _ => false
    }
}

impl<'a> Validator<'a> {
    pub fn new(ion: &'a Ion, spec: &'a Ion) -> Validator<'a> {
        Validator { ion: ion, spec: spec, errors: Vec::new() }
    }

    pub fn validate(mut self) -> Option<Vec<ValidationError>> {

        for (name, sect_spec) in self.spec.iter() {
            if name.starts_with("ION::") {
                let (_, sect_name) = name.split_at(5);

                self.section(sect_name, sect_spec);
            }
        }

        if self.errors.len() > 0 {
            Some(self.errors)
        } else {
            None
        }
    }

    fn section(&mut self, name: &str, spec: &Section) {
        println!("[{}]", name);
        let target = match self.ion.get(name) {
            Some(s) => s,
            None => {
                self.errors.push(ValidationError { kind: ErrorKind::Section, desc: format!("target section {} not found", name)});
                return
            }
        };

        let (dict, _table) : (Vec<_>, Vec<_>) = spec.dictionary.iter().partition(dict_table);

        println!("{:?}", dict);
        for (name, spec) in dict {
            let field = match (*target).get(name) {
                Some(f) => f,
                None => {
                    self.errors.push(ValidationError { kind: ErrorKind::Field, desc: format!("target field {} not found", name)});
                    continue
                }
            };
            self.field(name, spec, field);
        }
    }

    fn field(&mut self, name: &str, spec: &Value, target: &Value) {
        let exp_type_str = type_str(spec);
        let act_type_str = target.type_str();

        if exp_type_str != act_type_str {
            self.errors.push(ValidationError { kind: ErrorKind::FieldInvalidType, desc: format!("target field {} has invalid type, expected {}, got {}", name, exp_type_str, act_type_str) });
        }
    }

        //             if let Some(tgt) = tgt_section.dictionary.get(name) {
        //                 let exp_type_str = type_str(spec.get("type"));
        //                 if tgt.type_str() !=  exp_type_str {
        //                     self.errors.push(ValidationError { desc: format!("field {}/{}, expected type {}, got {}", sect_name, name, exp_type_str, tgt.type_str())});
        //                 }
        //             } else {
        //                 self.errors.push(ValidationError { desc: format!("field {}/{} not found", sect_name, name)});
        //             }
        //         }
        //     }
        // }
}

#[derive(Debug)]
pub struct ValidationError {
    desc: String,
    // section: String,
    // field: Option<String>,
    kind: ErrorKind
}

#[derive(Debug)]
pub enum ErrorKind {
    Section,
    Field,
    FieldInvalidType
}

impl error::Error for ValidationError {
    fn description(&self) -> &str {
        &self.desc
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.desc.fmt(f)
    }
}

fn type_str(v: &Value) -> String {
    match v.get("type") {
        Some(&Value::String(ref v)) => v.to_owned(),
        _ => "string".to_owned()
    }
}
