use std::{fmt::Debug, hash::Hash};

use leptos_reactive::SignalGet;

use crate::view::View;

use super::{build_fucking_simple_tree, SuperFuckingBasicTreeNode};

pub fn debug_view<S, N, K>(tree_node: S) -> impl View
where
    N: SignalGet<S> + 'static,
    S: SuperFuckingBasicTreeNode<Item = N> + 'static,
    // K: Debug + Hash + Eq + 'static,
{
    build_fucking_simple_tree::<S, N, K>(tree_node)
}
