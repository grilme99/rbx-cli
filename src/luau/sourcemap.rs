use std::{cell::RefCell, fs, io::Cursor, path::Path, rc::Rc};

use anyhow::Context;
use serde::Deserialize;

pub type SourcemapNode = Rc<RefCell<InnerSourcemapNode>>;

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum InstanceClass {
    Script,
    LocalScript,
    ModuleScript,
    Folder,
}

impl ToString for InstanceClass {
    fn to_string(&self) -> String {
        let str = match self {
            InstanceClass::Folder => "Folder",
            InstanceClass::ModuleScript => "ModuleScript",
            InstanceClass::LocalScript => "LocalScript",
            InstanceClass::Script => "Script",
        };

        str.to_owned()
    }
}

/// Represents a Rojo sourcemap, with extremely easy traversal via the `parent` property.
///
/// ## Note on implementation
/// This tree implementation uses Rust's `Rc` and `RefCell` to make usage so simple. However, there might be a certain
/// overhead to doing this and the implementation may need to be changed later for performance reason.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InnerSourcemapNode {
    pub name: String,
    #[serde(rename = "className")]
    pub instance_class: InstanceClass,
    pub parent: Option<SourcemapNode>,
    pub file_paths: Option<Vec<String>>,
    pub children: Option<Vec<SourcemapNode>>,
}

#[allow(dead_code)]
impl InnerSourcemapNode {
    pub fn new_from_path(path: &Path) -> anyhow::Result<SourcemapNode> {
        let contents = fs::read(path).context(format!("Failed to read path {path:?}"))?;

        let root_node: SourcemapNode = serde_json::from_reader(Cursor::new(contents)).context(
            format!("Failed to parse path contents to sourcemap node {path:?}"),
        )?;

        let mut root_ref = root_node.borrow_mut();

        // Recursively populate the parent field of each node in the sourcemap
        if let Some(children) = &mut root_ref.children {
            for mut node in children {
                populate_parent_field(&mut node, Some(root_node.clone()));
            }
        }

        Ok(root_node.clone())
    }

    pub fn find_first_child(&self, child_name: &str) -> Option<SourcemapNode> {
        if let Some(children) = &self.children {
            for child in children {
                let child_ref = child.borrow();
                if child_ref.name == child_name {
                    return Some(child.clone());
                }
            }
        }

        None
    }
}

fn populate_parent_field(node: &mut SourcemapNode, parent: Option<SourcemapNode>) {
    let mut node_mut = node.borrow_mut();
    node_mut.parent = parent;

    if let Some(children) = &mut node_mut.children {
        for mut child in children {
            populate_parent_field(&mut child, Some(node.clone()));
        }
    }
}
