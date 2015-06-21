use { Ion, Value };

pub struct Validator<'a> {
    ion: &'a Ion,
    spec: &'a Ion,
    errors: Vec<ValidationError>
}

impl<'a> Validator<'a> {
    pub fn new(ion: &'a Ion, spec: &'a Ion) -> Validator<'a> {
        Validator { ion: ion, spec: spec, errors: Vec::new() }
    }

    pub fn validate(mut self) -> Option<Vec<ValidationError>> {

        for (name, spec_section) in self.spec.iter() {
            if name.starts_with("ION::") {
                let (_, sect_name) = name.split_at(5);
                println!("[{}]", sect_name);
                println!("{}", self.ion);
                let tgt_section = self.ion.get(sect_name).expect("tgt_section"); //TODO: add error

                for (name, spec) in &spec_section.dictionary {
                    println!("{}: {}", name, spec);
                    let spec = match spec.as_dictionary() {
                        Some(spec) => spec,
                        _ => {
                            self.errors.push(ValidationError { desc: format!("field spec for {} is invalid", name)});
                            continue
                        }
                    };

                    if let Some(tgt) = tgt_section.dictionary.get(name) {
                        let exp_type_str = type_str(spec.get("type"));
                        if tgt.type_str() !=  exp_type_str {
                            self.errors.push(ValidationError { desc: format!("field {}/{}, expected type {}, got {}", sect_name, name, exp_type_str, tgt.type_str())});
                        }
                    } else {
                        self.errors.push(ValidationError { desc: format!("field {}/{} not found", sect_name, name)});
                    }
                }
            }
        }

        if self.errors.len() > 0 {
            Some(self.errors)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct ValidationError {
    pub desc: String
}

fn type_str(v: Option<&Value>) -> String {
    match v.map(Value::as_string) {
        Some(Some(v)) => v.to_owned(),
        _ => "string".to_owned()
    }
}
