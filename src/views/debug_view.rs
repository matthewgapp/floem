use crate::view::View;

use super::{build_fucking_simple_tree, SuperFuckingBasicTreeNode};

pub fn debug_view<S>(tree_node: S) -> impl View
where
    S: SuperFuckingBasicTreeNode<Item = S> + 'static,
{
    build_fucking_simple_tree(tree_node)
}
