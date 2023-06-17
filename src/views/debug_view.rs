use leptos_reactive::SignalGet;

use crate::view::View;

use super::{build_fucking_simple_tree, SuperFuckingBasicTreeNode};

pub fn debug_view<S, N>(tree_node: S) -> impl View
where
    N: SignalGet<S> + 'static,
    S: SuperFuckingBasicTreeNode<Item = N> + 'static,
{
    build_fucking_simple_tree(tree_node)
}
