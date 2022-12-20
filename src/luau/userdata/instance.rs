use std::{cell::RefCell, rc::Rc};

use mlua::{MetaMethod, UserData};

use crate::luau::sourcemap::{InstanceClass, SourcemapNode};

pub type Instance = Rc<RefCell<InnerInstance>>;

/// *Very* thin API over Roblox's [`Instance`](https://create.roblox.com/docs/reference/engine/datatypes/Instance)
/// class. The important thing here is that all instances known to rbx-cli *must* have a corresponding entry in the
/// Rojo sourcemap. It is expected that the only Instances that will be referenced are Roblox `Folder`s (directories)
/// and any `Script` class (files).
///
/// An error will be thrown when constructing a new Instance if a corresponding sourcemap entry can not be found. As a
/// result, rbx-cli works best with projects fully-managed by Rojo.
#[derive(Debug, Clone)]
pub struct InnerInstance {
    pub name: String,
    pub instance_class: InstanceClass,
    pub sourcemap_node: SourcemapNode,
    pub parent: Option<Instance>,
}

impl InnerInstance {
    pub fn new(sourcemap_node: SourcemapNode, parent: Option<Instance>) -> Instance {
        let sourcemap_ref = sourcemap_node.borrow();
        let name = sourcemap_ref.name.to_owned();
        let instance_class = sourcemap_ref.instance_class.to_owned();

        // Explicitly drop the ref so we can move ownership of the sourcemap node
        drop(sourcemap_ref);

        Rc::new(RefCell::new(Self {
            name,
            instance_class,
            sourcemap_node,
            parent,
        }))
    }
}

impl UserData for InnerInstance {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        // TODO: How can we remove the extra allocation when returning Instance properties?
        fields.add_field_method_get("Name", |_, instance| Ok(instance.name.to_owned()));
        fields.add_field_method_get("Parent", |_, instance| {
            if let Some(parent) = &instance.parent {
                Ok(Some(parent.clone()))
            } else {
                Ok(None)
            }
        });

        fields.add_field_method_get("ClassName", |_, _| Ok("Instance"));

        // Roblox deviation: `__tostring` normally returns the Instance name, but we can't get that with mlua.
        fields.add_meta_field_with(MetaMethod::ToString, |_| Ok("Instance"));
    }

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        // Indexing children
        methods.add_meta_method(MetaMethod::Index, |_, instance, child_name: String| {
            let sourcemap_ref = instance.sourcemap_node.borrow();

            if let Some(child_node) = sourcemap_ref.find_first_child(&child_name) {
                // TODO: How can we return the instance without constructing a whole new struct and allocating more memory?
                let parent = Rc::new(RefCell::new(instance.clone()));
                let child_instance =
                    Rc::new(RefCell::new(InnerInstance::new(child_node, Some(parent))));

                return Ok(Some(child_instance));
            } else {
                // Indexed child does not exist
                let class_name = instance.instance_class.to_string();
                // TODO: Implement `GetFullName` Roblox functionality
                let instance_full_name = instance.name.to_owned();

                return Err(mlua::Error::RuntimeError(format!(
                    "{child_name} is not a valid member of {class_name} \"{instance_full_name}\""
                )));
            }
        });
    }
}
