use std::collections::HashMap;

#[derive(Debug, PartialOrd, PartialEq)]
pub enum PreTreeErr {
    MethodDisallow(String),
    Duplicated(String),
    Ambiguity(String),
    Unknow(String),
}

/// thank ['preTree']
///
/// ['preTree']: https://crates.io/crates/pretree
#[derive(Default, Debug)]
pub struct PreTree {
    tree_group: HashMap<String, Tree>,
}

impl PreTree {
    pub fn new() -> Self {
        let mut p = PreTree::default();
        let methods = [
            "GET", "HEAD", "POST", "PUT", "PATCH", "DELETE", "CONNECT", "OPTIONS", "TRACE",
        ];

        for method in methods {
            let tree = Tree::new(method);
            p.tree_group.insert(method.to_string(), tree);
        }

        p
    }

    /// Store routing rules
    ///
    /// 存储路由规则
    ///
    /// # Parameters
    ///
    /// * `method` - HTTP method, such as GET, POST,DELETE ...
    ///
    /// * `url_rule` - url routing rule, such as  /user/:id
    /// # Example
    /// ```
    /// use pretree::Pretree;
    /// let mut p = Pretree::new();
    /// p.store("GET","account/{id}/info/:name");
    /// p.store("GET","account/:id/login");
    /// p.store("GET","account/{id}");
    /// p.store("GET","bacteria/count_number_by_month");
    /// ```
    pub fn store(&mut self, method: &str, url_rule: &str) -> Result<(), PreTreeErr> {
        if let Some(t) = self.tree_group.get_mut(method) {
            t.insert(url_rule)
        } else {
            Err(PreTreeErr::MethodDisallow(method.into()))
        }
    }

    /// Query the tree node with matching URL and return variables
    ///
    /// 查询URL匹配的树节点并返回变量
    /// # Parameters
    ///
    /// * `method` - HTTP method, such as GET, POST,DELETE ...
    ///
    /// * `url_path` - URL path to access, such as account/929239
    ///
    /// # Results
    /// * bool -  Does it exist
    /// * String - url routing rule
    /// * HashMap<String, String> - Routing variables
    /// ```
    /// use algorithms::pretree::PreTree;
    /// let mut p = PreTree::new();
    /// p.store("GET","account/{id}/info/:name");
    /// p.store("GET","account/:id/login");
    /// p.store("GET","account/{id}");
    /// p.store("GET","bacteria/count_number_by_month");
    /// if let Some((rule, vars)) = p.query("GET","account/929239") {
    ///    println!("rule:{} vars:{:#?}",rule,vars);
    ///    assert_eq!(rule,"account/{id}");
    ///    assert_eq!(vars.get("id"),Some(&"929239".to_string()));
    /// }
    /// ```
    pub fn query(
        &self,
        method: &str,
        url_path: &str,
    ) -> Option<(&String, HashMap<String, String>)> {
        self.tree_group
            .get(method)
            .and_then(|t| t.search(url_path))
            .filter(|(t, _)| t.is_end)
            .map(|(t, vars)| (&t.rule, vars))
    }
}

#[derive(Debug)]
struct Tree {
    rule: String,
    name: String,
    /// 如果有很多节点，换map会有效果吗?
    nodes: Vec<Tree>,
    is_end: bool,
    is_variable: bool,
}

impl Tree {
    fn new(name: &str) -> Self {
        Tree {
            rule: String::from(""),
            name: name.to_string(),
            nodes: vec![],
            is_end: false,
            is_variable: false,
        }
    }

    fn variable(name: &str, is_variable: bool) -> Tree {
        Tree {
            rule: String::from(""),
            name: name.to_string(),
            nodes: vec![],
            is_end: false,
            is_variable,
        }
    }

    /// Get the variable name of the current node
    ///
    /// 获取当前节点的变量名
    pub fn var_name(&self) -> String {
        self.name.trim_start_matches(':').to_string()
    }

    /// 目前没有处理重复路径
    fn insert(&mut self, url_rule: &str) -> Result<(), PreTreeErr> {
        let list = parse_rule(url_rule);
        let mut cursor = Some(self);
        for word in list.iter() {
            if let Some(tree) = cursor.take() {
                let mut index = None;
                let mut has_var = false;
                for (idx, t) in tree.nodes.iter().enumerate() {
                    has_var |= t.is_variable;
                    if &t.name == word {
                        index = Some(idx);
                    }
                }

                //同一层只能有一个路径参数
                if index.is_none() && has_var && is_variable(word) {
                    return Err(PreTreeErr::Ambiguity(url_rule.into()));
                }

                if let Some(t) = index {
                    cursor = tree.nodes.get_mut(t);
                } else {
                    let node = Tree::variable(word, is_variable(word));
                    tree.nodes.push(node);
                    cursor = tree.nodes.last_mut();
                }
            } else {
                break;
            }
        }

        if let Some(tree) = cursor {
            if tree.is_end {
                Err(PreTreeErr::Duplicated(url_rule.into()))
            } else {
                tree.rule = url_rule.into();
                tree.is_end = true;
                Ok(())
            }
        } else {
            Err(PreTreeErr::Unknow(url_rule.into()))
        }
    }

    fn search(&self, url_path: &str) -> Option<(&Tree, HashMap<String, String>)> {
        let mut vars = HashMap::new();
        let mut cursor = Some(self);
        let list = parse_path(url_path);

        for (index, word) in list.into_iter().enumerate() {
            if let Some(tree) = cursor.take() {
                //尝试直接匹配
                for t in tree.nodes.iter() {
                    if t.name == word {
                        cursor.replace(t);
                        break;
                    }
                }

                //没找到，看下是否有变量。第一个词不能是变量
                if cursor.is_none() && 0 < index {
                    for t in tree.nodes.iter() {
                        if t.is_variable {
                            vars.insert(t.var_name(), word);
                            cursor.replace(t);
                            break;
                        }
                    }
                }
            } else {
                break;
            }
        }

        cursor.map(|tree| (tree, vars))
    }
}

fn parse_rule(path: &str) -> Vec<String> {
    let new_path = format_rule(path);
    let paths: Vec<String> = new_path
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_owned())
        .collect();
    paths
}

fn parse_path(path: &str) -> Vec<String> {
    let paths: Vec<String> = path
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_owned())
        .collect();
    paths
}
fn format_rule(rule: &str) -> String {
    rule.replace('{', ":").replace('}', "")
}

fn is_variable(s: &str) -> bool {
    s.starts_with(':')
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::pretree::PreTreeErr;

    #[test]
    fn test_sotre() {
        use super::PreTree;

        let mut p = PreTree::new();
        assert!(p.store("GET", "/pet").is_ok());
        assert!(p.store("GET", "/pet/hi").is_ok());
        assert_eq!(
            Err(PreTreeErr::Duplicated(String::from("/pet"))),
            p.store("GET", "/pet")
        );

        assert!(p.store("GET", "/pet/{petId}/").is_ok());
        assert!(p.store("GET", "/pet/2/").is_ok());
        assert_eq!(
            Err(PreTreeErr::Duplicated(String::from("/pet/{petId}/"))),
            p.store("GET", "/pet/{petId}/")
        );
        assert_eq!(
            Err(PreTreeErr::Ambiguity(String::from("/pet/{petName}/"))),
            p.store("GET", "/pet/{petName}/")
        );
        assert!(p.store("GET", "/pet/{petId}/{petName}").is_ok());
        assert_eq!(
            Err(PreTreeErr::Ambiguity(String::from("/pet/{petId}/{petSex}"))),
            p.store("GET", "/pet/{petId}/{petSex}")
        );
        assert_eq!(
            Err(PreTreeErr::Ambiguity(String::from(
                "/pet/{petName}/{petSex}"
            ))),
            p.store("GET", "/pet/{petName}/{petSex}")
        );
        assert!(p.store("GET", "/pet/{petId}/info").is_ok());
        assert!(p.store("GET", "/pet/{petId}/test").is_ok());
        assert_eq!(
            Err(PreTreeErr::Duplicated(String::from("/pet/{petId}/info"))),
            p.store("GET", "/pet/{petId}/info")
        );
    }

    #[test]
    fn test_match() {
        use super::PreTree;

        let data: [[&str; 3]; 10] = [
            ["GET", "/pet", "/pet"],
            ["GET", "/pet/hi", "/pet/hi"],
            ["GET", "/pet/{petId}", "/pet/1"],
            ["POST", "/pet/{petId}", "/pet/2"],
            ["POST", "/pet/3", "/pet/3"],
            ["POST", "/pet/{petId}/uploadImage", "/pet/1/uploadImage"],
            ["POST", "/pet/{petId}/info", "/pet/1/info"],
            [
                "POST",
                "/pet/{petId}/{petName}/{petSex}/info",
                "/pet/1/z/girl/info",
            ],
            [
                "POST",
                "/test/{petId}/{petName}/{petSex}/info",
                "/test/1/z/girl/info",
            ],
            ["GET", "/store/inventory", "/store/inventory"],
        ];

        let mut p = PreTree::new();

        for v in data {
            let method = v[0];
            let source_rule = v[1];
            let res = p.store(method, source_rule);
            assert_eq!(
                true,
                res.is_ok(),
                "store method:{:?}, err:{:?}",
                method,
                res,
            );
        }

        assert!(p.query("GET", "/pets").is_none());
        assert!(p.query("GET", "/store").is_none());
        assert!(p.query("POST", "/pet/{petId}/info/xxxx").is_none());

        for v in data {
            let method = v[0];
            let source_rule = v[1];
            let url_path = v[2];
            let (rule, vars) = p.query(method, url_path).unwrap();
            println!("rule:{:?} vars:{:?}", rule, vars);
            assert_eq!(source_rule, rule);

            match url_path {
                "/pet/1" => {
                    assert_eq!(Some(&String::from("1")), vars.get("petId"));
                }
                "/pet/2" => {
                    assert_eq!(Some(&String::from("2")), vars.get("petId"));
                }
                "/pet/1/uploadImage" | "/pet/1/info" => {
                    assert_eq!(Some(&String::from("1")), vars.get("petId"));
                }
                "/pet/1/z/girl/info" | "/test/1/z/girl/info" => {
                    let mut map = HashMap::new();
                    map.insert("petName".into(), "z".into());
                    map.insert("petSex".into(), "girl".into());
                    map.insert("petId".into(), "1".into());

                    assert_eq!(map, vars);
                }
                _ => assert!(vars.is_empty(), "{:?}", url_path),
            }
        }
    }
}
