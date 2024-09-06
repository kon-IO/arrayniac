use std::{cell::RefCell, collections::HashMap, rc::Rc};

use sonic_rs::{JsonContainerTrait, JsonType, JsonValueTrait, Number, Value};

#[derive(Debug)]
enum JsonValue<'a> {
    Null,
    Boolean(bool),
    String(&'a str),
    Number(Number),
    Object(HashMap<&'a str, Node<'a>>),
    Array(Vec<Node<'a>>),
}

impl JsonValue<'_> {
    #[allow(dead_code)]
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
pub struct Node<'a> {
    ind: Option<usize>,
    val: JsonValue<'a>,
}

impl Node<'_> {
    fn set_ind(&mut self, ind: usize) {
        self.ind = Some(ind);
    }
}

pub type ObjectVariants<'a> = Vec<Rc<Vec<&'a str>>>;

pub struct Json<'a> {
    path: RefCell<String>,
    indent: usize,
    info_print: bool,
    variant_map: HashMap<String, ObjectVariants<'a>>,
    res: Option<Node<'a>>,
}

impl<'a> Json<'a> {
    fn new(info_print: bool, root: &'a Value) -> Json<'a> {
        let mut n = Json {
            path: RefCell::new(String::new()),
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
        let Some(ref n) = self.res else {
            return None;
        };

        Some((self.to_string_node(n), self.to_string_variant()))
    }

    fn to_string_variant(&self) -> String {
        let mut out_map: HashMap<&str, Value> = HashMap::new();
        for (path, variants) in &self.variant_map {
            if variants.len() == 1 {
                let mut var_map = HashMap::new();
                variants
                    .get(0)
                    .unwrap()
                    .iter()
                    .enumerate()
                    .for_each(|(ind, name)| {
                        var_map.insert(name, ind);
                    });

                out_map.insert(path.as_str(), sonic_rs::to_value(&var_map).unwrap());
                continue;
            }
            let mut var_arr = Vec::new();
            for var in variants {
                let mut var_map = HashMap::new();
                var.iter().enumerate().for_each(|(ind, name)| {
                    var_map.insert(*name, ind);
                });
                var_arr.push(var_map);
            }
            out_map.insert(path, sonic_rs::to_value(&var_arr).unwrap());
        }
        sonic_rs::to_string(&out_map).unwrap()
    }

    fn to_string_node(&self, node: &Node) -> String {
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
                {
                    let mut path = self.path.borrow_mut();
                    path.push_str("[]");
                }
                let str_arr: Vec<String> = arr
                    .into_iter()
                    .map(|v| {
                        let val = self.to_string_node(&v);
                        val
                    })
                    .collect();
                let mut path = self.path.borrow_mut();
                let len = path.len();
                path.truncate(len - 2);
                // TODO: This is BAD
                let mut out_str = str_arr.join(",");
                out_str.insert(0, '[');
                out_str.push(']');
                out_str
            }
            JsonValue::Object(o) => {
                let (variants_len, variant);
                let mut s = "[".to_owned();
                {
                    let path = self.path.borrow_mut();
                    let ind = node.ind.expect("Expected object to have a variant index");
                    (variants_len, variant) = self.get_variant(&path, ind);
                    if variants_len != 1 {
                        s.push_str(&ind.to_string());
                        s.push(',');
                    }
                }

                for k in variant.iter() {
                    let (_, val) = o
                        .iter()
                        .find(|(key, _)| **key == *k)
                        .expect("Expect to find key");

                    {
                        let mut path = self.path.borrow_mut();
                        path.push('.');
                        path.push_str(&k);
                    }
                    s.push_str(self.to_string_node(val).as_str());
                    s.push_str(",");
                    let mut path = self.path.borrow_mut();
                    let len = path.len();
                    path.truncate(len - k.len() - 1);
                }
                // Remove last comma
                s.truncate(s.len() - 1);
                s.push(']');
                s
            }
        }
    }

    fn get_variant(&self, path: &String, ind: usize) -> (usize, Rc<Vec<&'a str>>) {
        let our_variants = self
            .variant_map
            .get(path.as_str())
            .expect("Expected variant vector to exist");
        (
            our_variants.len(),
            Rc::clone(
                &our_variants
                    .get(ind)
                    .expect("Expected object variant to exist"),
            ),
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
                val: JsonValue::String(root.as_str().unwrap()),
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

    fn parse_array(&mut self, node: &'a Value, array_node: &mut Node<'a>) {
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
                    val: JsonValue::String(v.as_str().unwrap()),
                }),
                JsonType::Array => {
                    let mut arr_node = Node {
                        ind: None,
                        val: JsonValue::Array(Vec::new()),
                    };
                    {
                        let p = self.path.get_mut();
                        p.push_str("[]");
                        if self.info_print {
                            println!("{: <1$}Going into path {2}", "", self.indent, p);
                        }
                    }
                    self.parse_array(v, &mut arr_node);
                    let p = self.path.get_mut();
                    if self.info_print {
                        println!("{: <1$}Going out of path {2}", "", self.indent, p);
                    }
                    p.truncate(p.len() - 2);
                    arr.push(arr_node);
                }
                JsonType::Object => {
                    let mut new_node = Node {
                        ind: None,
                        val: JsonValue::Object(HashMap::new()),
                    };
                    {
                        let p = self.path.get_mut();
                        p.push_str("[]");
                        if self.info_print {
                            println!("{: <1$}Going into path {2}", "", self.indent, p);
                        }
                    }
                    let ind = self.parse_object(v, &mut new_node);
                    {
                        if self.info_print {
                            println!(
                                "{: <1$}Going out of path {2}",
                                "",
                                self.indent,
                                self.path.borrow()
                            );
                        }
                    }
                    let p = self.path.get_mut();
                    let len = p.len();
                    p.truncate(len - 2);
                    new_node.set_ind(ind);
                    arr.push(new_node);
                }
            }
        }
    }
    fn parse_object(&mut self, node: &'a Value, obj_node: &mut Node<'a>) -> usize {
        assert!(node.get_type() == JsonType::Object);

        let JsonValue::Object(map) = &mut obj_node.val else {
            panic!("Expected node to be of object type");
        };

        let mut obj_type = Vec::new();

        node.as_object().unwrap().iter().for_each(|(k, v)| {
            obj_type.push(k);
            match v.get_type() {
                JsonType::Boolean => {
                    map.insert(
                        k,
                        Node {
                            ind: None,
                            val: JsonValue::Boolean(v.as_bool().unwrap()),
                        },
                    );
                }
                JsonType::Null => {
                    map.insert(
                        k,
                        Node {
                            ind: None,
                            val: JsonValue::Null,
                        },
                    );
                }
                JsonType::Number => {
                    map.insert(
                        k,
                        Node {
                            ind: None,
                            val: JsonValue::Number(v.as_number().unwrap()),
                        },
                    );
                }
                JsonType::String => {
                    map.insert(
                        k,
                        Node {
                            ind: None,
                            val: JsonValue::String(v.as_str().unwrap()),
                        },
                    );
                }
                JsonType::Array => {
                    let mut arr_node = Node {
                        ind: None,
                        val: JsonValue::Array(Vec::new()),
                    };
                    {
                        let p = self.path.get_mut();
                        p.push('.');
                        p.push_str(k);
                        if self.info_print {
                            println!("{: <1$}Going into path {2}", "", self.indent, p);
                        }
                    }
                    self.parse_array(v, &mut arr_node);
                    let p = self.path.get_mut();
                    if self.info_print {
                        println!("{: <1$}Going out of path {2}", "", self.indent, p);
                    }
                    p.truncate(p.len() - k.len() - 1);
                    map.insert(k, arr_node);
                }
                JsonType::Object => {
                    let mut new_node = Node {
                        ind: None,
                        val: JsonValue::Object(HashMap::new()),
                    };
                    {
                        let p = self.path.get_mut();
                        p.push('.');
                        p.push_str(k);
                        if self.info_print {
                            println!("{: <1$}Going into path {2}", "", self.indent, p);
                        }
                    }
                    let ind = self.parse_object(v, &mut new_node);
                    let p = self.path.get_mut();
                    if self.info_print {
                        println!("{: <1$}Going out of path {2}", "", self.indent, p);
                    }
                    p.truncate(p.len() - k.len() - 1);
                    new_node.set_ind(ind);
                    map.insert(k, new_node);
                }
            };
        });

        obj_type.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p = self.path.get_mut();
        if self.variant_map.get(p.as_str()).is_none() {
            self.variant_map.insert(p.clone(), Vec::new());
        };
        let var_vec = self.variant_map.get_mut(p.as_str()).unwrap();
        let variant_ind;
        if let Some(existing) = var_vec.iter().position(|v| **v == obj_type) {
            variant_ind = existing;
        } else {
            if self.info_print {
                println!(
                    "{: <1$}Found new variant in {2}: {obj_type:?}",
                    "", self.indent, p
                );
            } else {
                println!("Found new variant in {}: {obj_type:?}", p);
            }
            var_vec.push(Rc::new(obj_type));
            variant_ind = var_vec.len() - 1;
        }
        variant_ind
    }
}

pub fn parse_root(root: &Value, info_print: bool) -> Json {
    let temp_parse = Json::new(info_print, root);
    temp_parse
}
