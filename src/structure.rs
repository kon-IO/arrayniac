use std::{collections::HashMap, sync::Arc};

use sonic_rs::{JsonContainerTrait, JsonType, JsonValueTrait, Number, Value};

// type Variants = Vec<TypeNode>;

#[derive(Debug)]
enum JsonValue {
    Null,
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
            JsonValue::Null => JsonType::Null,
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
pub struct ObjectVariant<'a> {
    variant: Vec<(&'a str, JsonType)>,
}

impl<'a> ObjectVariant<'a> {
    fn new(variant: Vec<(&'a str, JsonType)>) -> ObjectVariant<'a> {
        return ObjectVariant { variant };
    }
}

pub type ObjectVariants<'a> = Vec<Arc<ObjectVariant<'a>>>;

pub struct Json<'a> {
    path: String,
    indent: usize,
    info_print: bool,
    variant_map: HashMap<String, ObjectVariants<'a>>,
    res: Option<Node>,
}

impl<'a> Json<'a> {
    fn new(info_print: bool, root: &'a Value) -> Json<'a> {
        let mut n = Json {
            path: String::new(),
            indent: 0,
            info_print: info_print,
            variant_map: HashMap::new(),
            res: None,
        };
        n.parse_root(root);
        n
    }

    pub fn get_root(&self) -> &Node {
        return self.res.as_ref().unwrap();
    }

    pub fn get_variants(&self) -> &HashMap<String, ObjectVariants<'a>> {
        return &self.variant_map;
    }

    pub fn to_string(&self) -> Option<(String, String)> {
        let Some(ref l) = self.res else {
            return None;
        };
        Some((
            self.to_string_node(&mut String::new(), l),
            self.to_string_variant(),
        ))
    }

    fn to_string_variant(&self) -> String {
        let mut out_map: HashMap<&str, Value> = HashMap::new();
        for (path, variants) in &self.variant_map {
            if variants.len() == 1 {
                let mut var_map = HashMap::new();
                variants
                    .get(0)
                    .unwrap()
                    .variant
                    .iter()
                    .enumerate()
                    .for_each(|(ind, (name, _))| {
                        var_map.insert(name, ind);
                    });

                out_map.insert(path.as_str(), sonic_rs::to_value(&var_map).unwrap());
                continue;
            }
            let mut var_arr = Vec::new();
            for var in variants {
                let mut var_map = HashMap::new();
                var.variant.iter().enumerate().for_each(|(ind, (name, _))| {
                    var_map.insert(*name, ind);
                });
                var_arr.push(var_map);
            }
            out_map.insert(path, sonic_rs::to_value(&var_arr).unwrap());
        }
        sonic_rs::to_string(&out_map).unwrap()
    }

    fn to_string_node(&'_ self, path: &mut String, node: &Node) -> String {
        match &node.val {
            JsonValue::Null => "null".to_owned(),
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
                        .find(|(key, _)| **key == *k)
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
        let our_variants = self
            .variant_map
            .get(path.as_str())
            .expect("Expected variant vector to exist");
        (
            our_variants.len(),
            our_variants
                .get(ind)
                .expect("Expected object variant to exist")
                .clone(),
        )
    }

    fn parse_root(&mut self, root: &'a Value) {
        let res = match root.get_type() {
            JsonType::Boolean => Node {
                ind: None,
                val: JsonValue::Boolean(root.as_bool().unwrap()),
            },
            JsonType::Null => Node {
                ind: None,
                val: JsonValue::Null,
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
                self.parse_array(root, &mut node);
                node
            }
            JsonType::Object => {
                let mut new_node = Node {
                    ind: None,
                    val: JsonValue::Object(HashMap::new()),
                };
                let ind = self.parse_object(root, &mut new_node);
                new_node.set_ind(ind);
                new_node
            }
        };
        self.res = Some(res);
    }

    fn parse_array(&'_ mut self, node: &'a Value, array_node: &mut Node) {
        assert!(node.get_type() == JsonType::Array);

        let JsonValue::Array(arr) = &mut array_node.val else {
            panic!("Expected node to be of array type");
        };

        for v in node.as_array().unwrap().iter() {
            match v.get_type() {
                JsonType::Null => arr.push(Node {
                    ind: None,
                    val: JsonValue::Null,
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
                    {
                        self.path.push_str("[]");
                    }
                    {
                        if self.info_print {
                            println!("{: <1$}Going into path {2}", "", self.indent, self.path);
                        }
                    }
                    {
                        self.parse_array(v, &mut arr_node);
                    }
                    {
                        if self.info_print {
                            println!("{: <1$}Going out of path {2}", "", self.indent, self.path);
                        }
                    }
                    self.path.truncate(self.path.len() - 2);
                    arr.push(arr_node);
                }
                JsonType::Object => {
                    let mut new_node = Node {
                        ind: None,
                        val: JsonValue::Object(HashMap::new()),
                    };
                    self.path.push_str("[]");
                    if self.info_print {
                        println!("{: <1$}Going into path {2}", "", self.indent, self.path);
                    }
                    let ind = self.parse_object(v, &mut new_node);
                    if self.info_print {
                        println!("{: <1$}Going out of path {2}", "", self.indent, self.path);
                    }
                    self.path.truncate(self.path.len() - 2);
                    new_node.set_ind(ind);
                    arr.push(new_node);
                }
            }
        }
    }
    fn parse_object(&'_ mut self, node: &'a Value, obj_node: &'_ mut Node) -> usize {
        assert!(node.get_type() == JsonType::Object);

        let JsonValue::Object(map) = &mut obj_node.val else {
            panic!("Expected node to be of object type");
        };

        let mut obj_type = Vec::new();

        node.as_object().unwrap().iter().for_each(|(k, v)| {
            obj_type.push((k, v.get_type()));
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
                            val: JsonValue::Null,
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
                    self.path.push('.');
                    self.path.push_str(k);
                    if self.info_print {
                        println!("{: <1$}Going into path {2}", "", self.indent, self.path);
                    }
                    self.parse_array(v, &mut arr_node);
                    if self.info_print {
                        println!("{: <1$}Going out of path {2}", "", self.indent, self.path);
                    }
                    self.path.truncate(self.path.len() - k.len() - 1);
                    map.insert(k.to_owned(), arr_node);
                }
                JsonType::Object => {
                    let mut new_node = Node {
                        ind: None,
                        val: JsonValue::Object(HashMap::new()),
                    };
                    self.path.push('.');
                    self.path.push_str(k);
                    if self.info_print {
                        println!("{: <1$}Going into path {2}", "", self.indent, self.path);
                    }
                    let ind = self.parse_object(v, &mut new_node);
                    if self.info_print {
                        println!("{: <1$}Going out of path {2}", "", self.indent, self.path);
                    }
                    self.path.truncate(self.path.len() - k.len() - 1);
                    new_node.set_ind(ind);
                    map.insert(k.to_owned(), new_node);
                }
            };
        });

        obj_type.sort_by(|a, b| a.0.partial_cmp(b.0).unwrap());
        if self.variant_map.get(self.path.as_str()).is_none() {
            self.variant_map.insert(self.path.clone(), Vec::new());
        };
        let var_vec = self.variant_map.get_mut(self.path.as_str()).unwrap();
        let variant_ind;
        if let Some(existing) = var_vec.iter().position(|v| v.variant == obj_type) {
            variant_ind = existing;
        } else {
            if self.info_print {
                println!("{: <1$}Found new variant: {obj_type:?}", "", self.indent);
            } else {
                println!("Found new variant: {obj_type:?}");
            }
            var_vec.push(Arc::new(ObjectVariant::new(obj_type)));
            variant_ind = var_vec.len() - 1;
        }
        variant_ind
    }
}

pub fn parse_root(root: &Value, info_print: bool) -> Json {
    let temp_parse = Json::new(info_print, root);
    temp_parse
}
