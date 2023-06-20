use leptos_reactive::{
    create_effect, create_rw_signal, ReadSignal, RwSignal, Scope, Signal, SignalGet,
    SignalGetUntracked, SignalUpdate,
};
use taffy::style::{FlexDirection, LengthPercentage, LengthPercentageAuto};
use vello::peniko::Color;

use crate::{id::Id, style::Style, view::View, ViewContext};

use super::{
    label, list, scroll,
    tree_builder::{tree_view, Children, Node, TreeView},
    virtual_list, Decorators, Label, List, VirtualList, VirtualListDirection, VirtualListItemSize,
    VirtualListVector,
};
use std::{collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData, ops::Deref, rc::Rc};

// needs a way to tell me to build children
// need to get a type that is is a closure that returns IntoIter<T> where T: SignalGet
// need a type that returns a closure that is |value: T|  -> V where V: View, T: SignalGet

pub trait TreeNode<T, V, I>
where
    I: IntoIterator + 'static,
    V: View + 'static,
    T: 'static,
{
    type Item: TreeNode<Self::Item, V, I> + 'static;
    type Children: SignalGet<I> + 'static;
    type K: Hash + Eq + 'static;
    type KeyFn: Fn(&Self::Item) -> Self::K + 'static;
    type ViewFn: Fn(&Self::Item) -> V + 'static;

    fn node(&self) -> Self::Item;
    fn has_children(&self) -> bool;
    fn children(&self) -> Self::Children;
    fn key_fn(&self) -> Self::KeyFn;
    fn view_fn(&self) -> Self::ViewFn;
}

// impl<N, T, V, I> TreeNode<T, V, I> for Rc<N>
// where
//     N: TreeNode<T, V, I>,
//     I: IntoIterator + 'static,
//     V: View + 'static,
//     T: 'static,
// {
//     type Children = N::Children;
//     type Item = N::Item;
//     type K = N::K;
//     type KeyFn = N::KeyFn;
//     type ViewFn = N::ViewFn;

//     fn children(&self) -> Self::Children {
//         self.as_ref().children()
//     }

//     fn has_children(&self) -> bool {
//         self.as_ref().has_children()
//     }

//     fn key(&self) -> Self::KeyFn {
//         self.as_ref().key()
//     }

//     fn node(&self) -> Self::Item {
//         self.as_ref().node()
//     }

//     fn view_fn(&self) -> Self::ViewFn {
//         self.as_ref().view_fn()
//     }
// }

// want a data structure that is a tree of signals

#[derive(Debug)]
struct ChildNode<T: 'static> {
    scope: Scope,
    data: RwSignal<T>,
}

impl<T: 'static> ChildNode<T> {
    fn new(scope: Scope, data: T) -> Self {
        Self {
            scope,
            data: create_rw_signal(scope, data),
        }
    }
    fn subscribe_effect<D: 'static>(&self, effect: impl Fn((T, Option<D>)) -> D + 'static)
    where
        T: Clone,
    {
        let data = self.data;
        create_effect(self.scope, move |prev| effect((data.get(), prev)))
    }
}

impl<T: 'static> Clone for ChildNode<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for ChildNode<T> {}

#[derive(Debug)]
pub struct ReactiveTree<T>
where
    T: std::fmt::Debug + 'static,
{
    scope: Scope,
    root: Id,
    nodes: RwSignal<HashMap<Id, ChildNode<T>>>,
    children: RwSignal<HashMap<Id, RwSignal<Vec<Id>>>>,
}

impl<T: Debug> Clone for ReactiveTree<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Debug> Copy for ReactiveTree<T> {}

impl<T> ReactiveTree<T>
where
    T: Debug,
{
    pub fn new(scope: Scope, root: Id, data: T) -> Self {
        let mut nodes = HashMap::new();
        nodes.insert(root, ChildNode::new(scope, data));
        Self {
            scope,
            root,
            nodes: create_rw_signal(scope, nodes),
            children: create_rw_signal(scope, HashMap::new()),
        }
    }

    pub fn has_untracked(&self, node: &Id) -> bool {
        self.nodes.get_untracked().get(node).is_some()
    }

    // TODO: fix up this function so that it's not dogshit
    pub fn insert_child(&self, parent: Id, child: Id, data: T) {
        println!("inserting child {:?}", child);

        let children = self.children.get_untracked();
        if let Some(parent_children) = children.get(&parent) {
            if parent_children
                .get_untracked()
                .iter()
                .find(|x| **x == child)
                .is_none()
            {
                parent_children.update(|parent_children| parent_children.push(child));
            }
        } else {
            self.children.update(|children| {
                children.insert(parent, create_rw_signal(self.scope, vec![child]));
            });
        }

        if self.nodes.get_untracked().get(&child).is_some() {
            println!("returning early with child {:?}", child);
            if self.nodes.get_untracked().get(&parent).is_none() {
                panic!("child set but parent wasn't");
            }
            return;
        }

        self.nodes.update(|nodes| {
            nodes.insert(child, ChildNode::new(self.scope, data));
        });

        self.nodes
            .get_untracked()
            .get(&parent)
            .unwrap()
            .data
            .update(|x| {
                // notify the parent;
            })
    }

    fn root_children(&self) -> RwSignal<Vec<Id>> {
        // we should be gucci to unwrap here
        *self
            .children
            .get_untracked()
            .get(&self.root)
            .unwrap_or(&create_rw_signal(self.scope, vec![]))
    }

    fn children_untracked(&self, id: &Id) -> Option<RwSignal<Vec<Id>>> {
        self.children.get_untracked().get(id).copied()
    }

    fn next_child_untracked(&self, parent: &Id, child: &Id) -> Option<Id> {
        if let Some(children) = self.children_untracked(parent) {
            let children = children.get_untracked();
            let index = children.iter().position(|c| c == child);
            index.and_then(|i| children.get(i + 1).map(|id| *id))
        } else {
            None
        }
    }

    fn next_child_from_root_untracked(&self, child: &Id) -> Option<Id>
    where
        T: Clone + Copy,
    {
        self.next_child_untracked(&self.root, child)
    }

    pub fn root_tree_node(&self) -> Signal<ConcreteTreeNode<T>>
    where
        T: Clone,
    {
        self.tree_node(&self.root).unwrap()
    }

    fn tree_node(&self, id: &Id) -> Option<Signal<ConcreteTreeNode<T>>>
    where
        T: Clone,
    {
        if self.nodes.get_untracked().get(id).is_none() {
            None
        } else {
            let nodes = self.nodes;
            let tree = *self;
            let scope = self.scope;
            let id = *id;
            Some(Signal::derive(self.scope, move || ConcreteTreeNode {
                scope,
                tree,
                id,
                value: Signal::derive(scope, move || {
                    // get untracked here because we don't want to respond to general changes in nodes when we build this signal
                    nodes.get_untracked().get(&id).unwrap().data.get()
                }),
            }))
        }
    }

    pub fn register_effect<D: 'static>(
        &self,
        id: Id,
        effect: impl Fn((T, Option<D>)) -> D + 'static,
    ) where
        T: Clone,
    {
        if let Some(node) = self.nodes.get_untracked().get(&id) {
            node.subscribe_effect(effect)
        } else {
            panic!("huh, there should have been a node");
        }
    }
}

pub struct ConcreteTreeNode<T>
where
    T: std::fmt::Debug + 'static,
{
    scope: Scope,
    tree: ReactiveTree<T>,
    id: Id,
    // TODO: do we need this?
    value: Signal<T>,
}

impl<T: Debug> Clone for ConcreteTreeNode<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Debug> Copy for ConcreteTreeNode<T> {}

// pub struct Hi<T: 'static + Debug> {
//     signal: Signal<T>,
//     tree: ReactiveTree<T>,
// }

// impl<T: Debug> Clone for Hi<T> {
//     fn clone(&self) -> Self {
//         *self
//     }
// }

// impl<T: Debug> Copy for Hi<T> {}

// impl <T> Copy for ConcreteTreeNode<T>;

pub struct TreeNodeIter<T: Debug + 'static> {
    scope: Scope,
    tree: ReactiveTree<T>,
    parent: Id,
    cur: Option<Id>,
}

impl<T: Debug> Clone for TreeNodeIter<T> {
    fn clone(&self) -> Self {
        TreeNodeIter {
            scope: self.scope,
            tree: self.tree,
            parent: self.parent,
            cur: self.cur.clone(),
        }
    }
}

type TreeNodeIterItem<T> = Signal<ConcreteTreeNode<T>>;

impl<T> Iterator for TreeNodeIter<T>
where
    T: Debug + Clone + 'static,
{
    type Item = TreeNodeIterItem<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cur) = self.cur {
            let next = self.tree.next_child_untracked(&self.parent, &cur);
            println!("unwrapping tree node {:?}", cur);
            println!("current tree {:#?}", self.tree.nodes.get_untracked());
            let item = self.tree.tree_node(&cur).unwrap();
            println!("current item: {:?}", item.get_untracked().id);
            println!("next item: {:?}", next);
            self.cur = next;
            Some(item)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
struct WrappedReactiveTree<S, T>(S, PhantomData<T>)
where
    T: Debug + 'static,
    S: SignalGet<ReactiveTree<T>> + SignalGetUntracked<ReactiveTree<T>>;

impl<S, T> Deref for WrappedReactiveTree<S, T>
where
    T: Debug + 'static,
    S: SignalGet<ReactiveTree<T>> + SignalGetUntracked<ReactiveTree<T>>,
{
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> IntoIterator for ConcreteTreeNode<T>
where
    T: Debug + Clone + 'static,
{
    type Item = TreeNodeIterItem<T>;
    type IntoIter = TreeNodeIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let first_child = self
            .tree
            .children_untracked(&self.id)
            .and_then(|children| children.get_untracked().first().map(|id| *id));
        TreeNodeIter {
            cur: first_child,
            parent: self.id,
            scope: self.scope,
            tree: self.tree,
        }
    }
}

pub trait DebugInfo {
    fn set_show_outline(&mut self, show: bool);
    fn show_outline(&self) -> bool;
}

impl<T> SuperFuckingBasicTreeNode for ConcreteTreeNode<T>
where
    T: Debug + Clone + 'static + DebugInfo,
{
    type I = ConcreteTreeNode<T>;
    type Children = Signal<Self::I>;
    type K = Id;
    type Item = TreeNodeIterItem<T>;
    type KeyFn = Box<dyn Fn(&Self::Item) -> Self::K>;
    type ViewFn = Box<dyn Fn(&Self::Item) -> Self::View>;
    type View = Label;

    fn children(&self) -> Self::Children {
        self.tree.tree_node(&self.id).unwrap()
    }

    fn has_children(&self) -> bool {
        self.tree
            .children
            .get()
            .get(&self.id)
            .map(|children| children.get().len() > 0)
            .unwrap_or_default()
    }

    fn key_fn(&self) -> Self::KeyFn {
        Box::new(|x| x.get().id)
    }

    fn node(&self) -> Self::Item {
        self.tree.tree_node(&self.id).unwrap()
    }

    fn view_fn(&self) -> Self::ViewFn {
        Box::new(|x| {
            let x = *x;
            let nodes = x.get_untracked().tree.nodes;
            let id = x.get_untracked().id;
            label(move || format!("Node with id {:?}", x.get().id))
                .on_event(crate::event::EventListener::PointerEnter, move |e| {
                    println!("updating id {:?} to outline: true", id);
                    if let Some(node) = nodes.get_untracked().get(&id) {
                        node.data.update(|d| d.set_show_outline(true))
                    } else {
                        panic!("no node found in event listener");
                    }
                    false
                })
                .on_event(crate::event::EventListener::PointerLeave, move |e| {
                    if let Some(node) = nodes.get_untracked().get(&id) {
                        node.data.update(|d| d.set_show_outline(false))
                    } else {
                        panic!("no node found in event listener");
                    }
                    false
                })
                .style(|| Style::BASE.background(Color::GREEN))
                .hover_style(|| Style::BASE.background(Color::RED))
        })
    }
}

// needs a way to tell me to build children
// need to get a type that is is a closure that returns IntoIter<T> where T: SignalGet
// need a type that returns a closure that is |value: T|  -> V where V: View, T: SignalGet

// i need a method to get something that can be turned into an iterator of children (wrapped in a signal). This iterator should return the same type as the node
// i need a method that can can return a closure
// i need a method that can return the current node

pub trait SuperFuckingBasicTreeNode: Copy {
    type View: View;
    type Item;
    type I: IntoIterator<Item = Self::Item>;
    type Children: SignalGet<Self::I>;
    type K: Hash + Eq + 'static;
    type KeyFn: Fn(&Self::Item) -> Self::K;
    type ViewFn: Fn(&Self::Item) -> Self::View;

    fn node(&self) -> Self::Item;
    fn has_children(&self) -> bool;
    fn children(&self) -> Self::Children;
    fn key_fn(&self) -> Self::KeyFn;
    fn view_fn(&self) -> Self::ViewFn;
}

pub struct NeverIterate<T>(std::marker::PhantomData<T>);

impl<T> IntoIterator for NeverIterate<T> {
    type IntoIter = std::iter::Empty<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::empty()
    }
}

pub fn build_fucking_simple_tree<S, N, K>(tree_node: S) -> TreeView<N, S::View>
where
    S: SuperFuckingBasicTreeNode<Item = N> + 'static,
    N: SignalGet<S>,
    // K: Debug + Hash + Eq + 'static,
{
    println!("build simple tree");
    let parent = Node::<N, _>::new(tree_node.view_fn());
    let children = move || {
        if tree_node.has_children() {
            Some(Children::new(
                move || {
                    let children = tree_node.children().get();
                    // println!("children len {}", children)
                    children
                },
                tree_node.key_fn(),
                move |item| build_fucking_simple_tree::<_, _, K>(item.get()),
            ))
        } else {
            None
        }
    };

    tree_view(tree_node.node(), parent, children)
        .style(|| Style::BASE.flex_direction(FlexDirection::Column))
    // .hover_style(|| Style::BASE.background(Color::REBECCA_PURPLE))
}
