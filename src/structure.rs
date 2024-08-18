use std::{collections::HashMap, sync::Arc};

use sonic_rs::{JsonContainerTrait, JsonType, JsonValueTrait, Number, Value};

// type Variants = Vec<TypeNode>;

#[derive(Debug)]
enum JsonValue {
    Null(()),
    Boolean(bool),
    String(String),
    Number(Number),
    Object(HashMap<String, Node>),
    Array(Vec<Node>),
}

impl JsonValue {
    fn json_type(&self) -> JsonType {
        match self {
            JsonValue::Array(_) => JsonType::Array,
            JsonValue::Boolean(_) => JsonType::Boolean,
            JsonValue::Null(_) => JsonType::Null,
            JsonValue::Number(_) => JsonType::Number,
            JsonValue::String(_) => JsonType::String,
            JsonValue::Object(_) => JsonType::Object,
        }
    }
}

#[derive(Debug)]
pub struct Node {
    ind: Option<usize>,
    val: JsonValue,
}

impl Node {
    fn set_ind(&mut self, ind: usize) {
        self.ind = Some(ind);
    }
}

#[derive(Debug)]
pub struct ObjectVariant {
    variant: Vec<(String, JsonType)>,
}

impl ObjectVariant {
    fn new(variant: Vec<(String, JsonType)>) -> ObjectVariant {
        return ObjectVariant { variant };
    }
}

pub type ObjectVariants = Vec<Arc<ObjectVariant>>;

#[derive(Debug)]
pub struct Json {
    root: Node,
    variants: HashMap<String, ObjectVariants>,
}

impl Json {
    pub fn get_root(&self) -> &Node {
        return &self.root;
    }

    pub fn get_variants(&self) -> &HashMap<String, ObjectVariants> {
        return &self.variants;
    }

    pub fn to_string(&self) -> (String, String) {
        (
            self.to_string_node(&mut String::new(), &self.root),
            self.to_string_variant(),
        )
    }

    fn to_string_variant(&self) -> String {
        let mut out_map: HashMap<String, Value> = HashMap::new();
        for (path, variants) in &self.variants {
            if variants.len() == 1 {
                let mut var_map = HashMap::new();
                variants
                    .get(0)
                    .unwrap()
                    .variant
                    .iter()
                    .enumerate()
                    .for_each(|(ind, (name, _))| {
                        var_map.insert(name.clone(), ind);
                    });

                out_map.insert(path.clone(), sonic_rs::to_value(&var_map).unwrap());
                continue;
            }
            let mut var_arr = Vec::new();
            for var in variants {
                let mut var_map = HashMap::new();
                var.variant.iter().enumerate().for_each(|(ind, (name, _))| {
                    var_map.insert(name.clone(), ind);
                });
                var_arr.push(var_map);
            }
            out_map.insert(path.clone(), sonic_rs::to_value(&var_arr).unwrap());
        }
        sonic_rs::to_string(&out_map).unwrap()
    }

    fn to_string_node(&self, path: &mut String, node: &Node) -> String {
        match &node.val {
            JsonValue::Null(_) => "null".to_owned(),
            JsonValue::Number(n) => sonic_rs::to_string(n).unwrap(),
            JsonValue::String(s) => sonic_rs::to_string(s).unwrap(),
            JsonValue::Boolean(b) => {
                if *b {
                    "true".to_owned()
                } else {
                    "false".to_owned()
                }
            }
            JsonValue::Array(arr) => {
                path.push_str("[]");
                let str_arr: Vec<String> = arr
                    .into_iter()
                    .map(|v| {
                        let val = self.to_string_node(path, &v);
                        val
                    })
                    .collect();
                path.truncate(path.len() - 2);
                // TODO: This is BAD
                let mut out_str = str_arr.join(",");
                out_str.insert(0, '[');
                out_str.push(']');
                out_str
            }
            JsonValue::Object(o) => {
                let ind = node.ind.expect("Expected object to have a variant index");
                let (variants_len, variant) = self.get_variant(path, ind);
                let mut s = "[".to_owned();
                if variants_len != 1 {
                    s.push_str(&ind.to_string());
                    s.push(',');
                }

                for (k, v) in variant.variant.iter() {
                    let (_, val) = o
                        .iter()
                        .find(|(key, _)| *key == k)
                        .expect("Expect to find key");

                    assert!(val.val.json_type() == *v);
                    path.push('.');
                    path.push_str(&k);
                    s.push_str(self.to_string_node(path, val).as_str());
                    s.push_str(",");
                    path.truncate(path.len() - k.len() - 1);
                }
                // Remove last comma
                s.truncate(s.len() - 1);
                s.push(']');
                s
            }
        }
    }

    fn get_variant(&self, path: &String, ind: usize) -> (usize, Arc<ObjectVariant>) {
        println!("Path: {}", path);
        let our_variants = self
            .variants
            .get(path)
            .expect("Expected variant vector to exist");
        (
            our_variants.len(),
            our_variants
                .get(ind)
                .expect("Expected object variant to exist")
                .clone(),
        )
    }
}

// struct TypeNode {
//     typ: JsonType,
//     path: String,
//     variants: Option<Variants>,
// }

// impl TypeNode {
//     fn new(path: String, typ: JsonType) -> TypeNode {
//         return TypeNode {
//             typ,
//             path,
//             variants: None,
//         };
//     }

//     pub fn create_from_root(root: &Value) -> TypeNode {
//         match root.get_type() {
//             JsonType::Array => {
//                 let t = TypeNode::new(String::new(), JsonType::Array);
//                 t.variants = Some(make_variants(root));
//                 t
//             }
//             JsonType::Boolean => TypeNode::new(String::new(), JsonType::Boolean),
//             JsonType::Null => TypeNode::new(String::new(), JsonType::Null),
//             JsonType::Number => TypeNode::new(String::new(), JsonType::Number),
//             JsonType::Object => {}
//             JsonType::String => TypeNode::new(String::new(), JsonType::String),
//         }
//     }
// }

// fn make_variants(of: &Value) -> Variants {
//     match of.get_type() {
//         bruh @ (JsonType::Boolean | JsonType::Null | JsonType::Number | JsonType::String) => vec![TypeNode::new(bruh)],
//     }
// }

pub fn parse_root(root: &Value) -> Json {
    let mut obj_map: HashMap<String, ObjectVariants> = HashMap::new();
    let res = match root.get_type() {
        JsonType::Boolean => Node {
            ind: None,
            val: JsonValue::Boolean(root.as_bool().unwrap()),
        },
        JsonType::Null => Node {
            ind: None,
            val: JsonValue::Null(()),
        },
        JsonType::String => Node {
            ind: None,
            val: JsonValue::String(root.as_str().unwrap().to_owned()),
        },
        JsonType::Number => Node {
            ind: None,
            val: JsonValue::Number(root.as_number().unwrap()),
        },
        JsonType::Array => {
            let mut node = Node {
                ind: None,
                val: JsonValue::Array(Vec::new()),
            };
            parse_array(root, &mut String::new(), &mut obj_map, &mut node);
            node
        }
        JsonType::Object => {
            let mut new_node = Node {
                ind: None,
                val: JsonValue::Object(HashMap::new()),
            };
            let ind = parse_object(root, &mut String::new(), &mut obj_map, &mut new_node);
            new_node.set_ind(ind);
            new_node
        }
    };
    Json {
        root: res,
        variants: obj_map,
    }
}

fn parse_array(
    node: &Value,
    path: &mut String,
    obj_map: &mut HashMap<String, ObjectVariants>,
    array_node: &mut Node,
) {
    assert!(node.get_type() == JsonType::Array);

    let JsonValue::Array(arr) = &mut array_node.val else {
        panic!("Expected node to be of array type");
    };

    node.as_array()
        .unwrap()
        .iter()
        .for_each(|v| match v.get_type() {
            JsonType::Null => arr.push(Node {
                ind: None,
                val: JsonValue::Null(()),
            }),
            JsonType::Boolean => arr.push(Node {
                ind: None,
                val: JsonValue::Boolean(v.as_bool().unwrap()),
            }),
            JsonType::Number => arr.push(Node {
                ind: None,
                val: JsonValue::Number(v.as_number().unwrap()),
            }),
            JsonType::String => arr.push(Node {
                ind: None,
                val: JsonValue::String(v.as_str().unwrap().to_owned()),
            }),
            JsonType::Array => {
                let mut arr_node = Node {
                    ind: None,
                    val: JsonValue::Array(Vec::new()),
                };
                path.push_str("[]");
                parse_array(v, path, obj_map, &mut arr_node);
                path.truncate(path.len() - 2);
                arr.push(arr_node);
            }
            JsonType::Object => {
                let mut new_node = Node {
                    ind: None,
                    val: JsonValue::Object(HashMap::new()),
                };
                path.push_str("[]");
                let ind = parse_object(v, path, obj_map, &mut new_node);
                path.truncate(path.len() - 2);
                new_node.set_ind(ind);
                arr.push(new_node);
            }
        });
}

fn parse_object(
    node: &Value,
    path: &mut String,
    obj_map: &mut HashMap<String, ObjectVariants>,
    obj_node: &mut Node,
) -> usize {
    assert!(node.get_type() == JsonType::Object);

    let JsonValue::Object(map) = &mut obj_node.val else {
        panic!("Expected node to be of object type");
    };

    let mut obj_type = Vec::new();

    node.as_object().unwrap().iter().for_each(|(k, v)| {
        obj_type.push((k.to_owned(), v.get_type()));
        match v.get_type() {
            JsonType::Boolean => {
                map.insert(
                    k.to_owned(),
                    Node {
                        ind: None,
                        val: JsonValue::Boolean(v.as_bool().unwrap()),
                    },
                );
            }
            JsonType::Null => {
                map.insert(
                    k.to_owned(),
                    Node {
                        ind: None,
                        val: JsonValue::Null(()),
                    },
                );
            }
            JsonType::Number => {
                map.insert(
                    k.to_owned(),
                    Node {
                        ind: None,
                        val: JsonValue::Number(v.as_number().unwrap()),
                    },
                );
            }
            JsonType::String => {
                map.insert(
                    k.to_owned(),
                    Node {
                        ind: None,
                        val: JsonValue::String(v.as_str().unwrap().to_owned()),
                    },
                );
            }
            JsonType::Array => {
                let mut arr_node = Node {
                    ind: None,
                    val: JsonValue::Array(Vec::new()),
                };
                path.push('.');
                path.push_str(k);
                parse_array(v, path, obj_map, &mut arr_node);
                path.truncate(path.len() - k.len() - 1);
                map.insert(k.to_owned(), arr_node);
            }
            JsonType::Object => {
                let mut new_node = Node {
                    ind: None,
                    val: JsonValue::Object(HashMap::new()),
                };
                path.push('.');
                path.push_str(k);
                let ind = parse_object(v, path, obj_map, &mut new_node);
                path.truncate(path.len() - k.len() - 1);
                new_node.set_ind(ind);
                map.insert(k.to_owned(), new_node);
            }
        };
    });

    obj_type.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    if obj_map.get(path).is_none() {
        obj_map.insert(path.clone(), Vec::new());
    };
    let var_vec = obj_map.get_mut(path).unwrap();
    let variant_ind;
    if let Some(existing) = var_vec.iter().position(|v| v.variant == obj_type) {
        variant_ind = existing;
    } else {
        var_vec.push(Arc::new(ObjectVariant::new(obj_type)));
        variant_ind = var_vec.len() - 1;
    }
    variant_ind
}
