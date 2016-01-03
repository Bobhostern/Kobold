// Represents a Kobold module, where all the structs go under, and a single thread of execution, with an optional "main" point that is executed if it is run
use std::collections::HashMap;
use super::ast::Expression;

#[derive(Clone, Debug)]
pub struct Module {
    name: String, // Struct complete name
    local_name: String, // Struct partial name
    module_code: Vec<Box<Expression>>,
}

impl Module {
    pub fn new(name: &str, module_code: Vec<Box<Expression>>) -> Module {
        Module {
            name: name.to_string(),
            local_name: name.clone().split('.').last().unwrap().to_string(),
            module_code: module_code
        }
    }

    pub fn get_full_name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Clone, Debug)]
struct ModuleNode {
    module: Option<Module>,
    map: Option<HashMap<String, ModuleNode>>
}

impl ModuleNode {
    fn new() -> ModuleNode {
        ModuleNode {
            module: None,
            map: None
        }
    }

    fn add_module(&mut self, name: &str, m: Module) {
        let key = name.clone().split('.').nth(0).unwrap();
        let new_name = name.clone().split('.').skip(1).collect::<String>();
        match self.map {
            Some(ref mut map) => {
                let was_entry = map.contains_key(key);
                let mut val = map.entry(key.to_string()).or_insert(ModuleNode::new());
                if new_name != "" {
                    val.add_module(&new_name, m)
                } else if was_entry {
                    panic!("Name already taken: {}", m.get_full_name());
                } else {
                    val.module = Some(m);
                }
            },
            None => {
                let mut map = HashMap::new();
                let mut sub = ModuleNode::new();
                if new_name != "" {
                    sub.add_module(&new_name, m);
                } else {
                    sub.module = Some(m);
                }
                map.insert(key.to_string(), sub);
                self.map = Some(map);
            }
        };
    }
}

#[derive(Debug)]
pub struct ModuleManager {
    module_map: ModuleNode
}

impl ModuleManager {
    pub fn new() -> ModuleManager {
        ModuleManager {
            module_map: ModuleNode::new(),
        }
    }

    pub fn add_module(&mut self, s: &str, m: Module) {
        self.module_map.add_module(s, m);
    }
}
