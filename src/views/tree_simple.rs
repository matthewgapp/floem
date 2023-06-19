use leptos_reactive::{
    create_rw_signal, ReadSignal, RwSignal, Scope, Signal, SignalGet, SignalGetUntracked,
    SignalUpdate,
};
use taffy::style::{FlexDirection, LengthPercentage, LengthPercentageAuto};

use crate::{id::Id, style::Style, view::View, ViewContext};

use super::{
    label, list, scroll,
    tree_builder::{tree_view, Children, Node, TreeView},
    virtual_list, Decorators, Label, List, VirtualList, VirtualListDirection, VirtualListItemSize,
    VirtualListVector,
};
use std::{
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    rc::Rc,
};

// a tree is a list of items, each of which is a function that returns a tree

// a tree is a a view made up of indented lists

// the list will take in some data and a function that returns a view

/*

Tree {
    Leaf {
        data: T,
        view_fn: Box<Fn(T) -> V>
    },
    Parent {
        each_fn: Fn() -> I + 'static where I: IntoIterator<Item = T>,
        key_fn: Fn(T) -> K + 'static where K: Hash + Eq,
        view_fn: Box<Fn(T) -> Tree>,
    }
}

 */

// struct Leaf<T, V> {
//     data: T,
//     view_fn: Box<Fn(T) -> V>,
// }

pub struct ListData<T, I, K, IF, KF, CVF>
where
    T: 'static,
    I: IntoIterator<Item = T> + 'static,
    K: Hash + Eq + 'static,
    IF: Fn() -> I + 'static,
    KF: Fn(&T) -> K + 'static,
    CVF: Fn(T) -> Box<dyn View> + 'static,
{
    pub each_fn: Box<IF>,
    pub key_fn: Box<KF>,
    pub view_fn: Box<CVF>,
    // pub phantom: std::marker::PhantomData<(T, I, K, V, PVF)>,
}

pub struct TreeProps<T, I, K, V, PVF, IF, KF, CVF>
where
    T: 'static,
    I: IntoIterator<Item = T> + 'static,
    K: Hash + Eq + 'static,
    V: View + 'static,
    IF: Fn() -> I + 'static,
    KF: Fn(&T) -> K + 'static,
    PVF: Fn(T) -> V + 'static,
    CVF: Fn(T) -> Box<dyn View> + 'static,
{
    pub data: T,
    pub parent_view_fn: Rc<Box<PVF>>,
    pub children: Option<ListData<T, I, K, IF, KF, CVF>>,
}

pub struct Tree<T, I, K, V, PVF, IF, KF, CVF>
where
    T: 'static,
    I: IntoIterator<Item = T> + 'static,
    K: Hash + Eq + 'static,
    V: View + 'static,
    IF: Fn() -> I + 'static,
    KF: Fn(&T) -> K + 'static,
    PVF: Fn(T) -> V + 'static,
    CVF: Fn(T) -> Box<dyn View> + 'static,
{
    id: Id,
    parent: V,
    children: Option<List<Box<dyn View>, CVF, T>>, // inner: TreeData<T, I, K, V, PVF, IF, KF, CVF>,
    phantom: std::marker::PhantomData<(T, I, K, V, PVF, IF, KF, CVF)>,
}

pub fn tree_simple<T, I, K, V, PVF, IF, KF, CVF>(
    props: TreeProps<T, I, K, V, PVF, IF, KF, CVF>,
) -> Tree<T, I, K, V, PVF, IF, KF, CVF>
where
    T: 'static,
    I: IntoIterator<Item = T> + 'static,
    K: Hash + Eq + 'static,
    V: View,
    IF: Fn() -> I + 'static,
    KF: Fn(&T) -> K + 'static,
    PVF: Fn(T) -> V + 'static,
    CVF: Fn(T) -> Box<dyn View>,
{
    let (id, (parent, children)) = ViewContext::new_id_with_child(|| {
        (
            (props.parent_view_fn.clone())(props.data),
            props.children.map(|list_data| {
                list(list_data.each_fn, list_data.key_fn, *list_data.view_fn).style(|| {
                    Style::BASE
                        .flex_direction(crate::style::FlexDirection::Column)
                        .margin_left(LengthPercentageAuto::Points(20.))
                })
            }),
        )
    });

    Tree {
        id,
        parent,
        children,
        phantom: std::marker::PhantomData,
    }
}

impl<T, I, K, V, PVF, IF, KF, CVF> View for Tree<T, I, K, V, PVF, IF, KF, CVF>
where
    T: 'static,
    I: IntoIterator<Item = T> + 'static,
    K: Hash + Eq + 'static,
    V: View + 'static,
    IF: Fn() -> I + 'static,
    KF: Fn(&T) -> K + 'static,
    PVF: Fn(T) -> V + 'static,
    CVF: Fn(T) -> Box<dyn View> + 'static,
{
    fn id(&self) -> Id {
        self.id
    }

    fn child(&mut self, id: Id) -> Option<&mut dyn View> {
        if self.parent.id() == id {
            Some(&mut self.parent)
        } else {
            self.children.as_mut().and_then(|children| {
                if children.id() == id {
                    Some(children as &mut dyn View)
                } else {
                    None
                }
            })
        }
    }

    fn children(&mut self) -> Vec<&mut dyn View> {
        let mut children = vec![&mut self.parent as &mut dyn View];
        if let Some(children_view) = self.children.as_mut() {
            children.push(children_view as &mut dyn View);
        }
        children
    }

    fn event(
        &mut self,
        cx: &mut crate::context::EventCx,
        id_path: Option<&[Id]>,
        event: crate::event::Event,
    ) -> bool {
        false
    }

    fn layout(&mut self, cx: &mut crate::context::LayoutCx) -> taffy::prelude::Node {
        cx.layout_node(self.id, true, |cx| {
            let mut nodes = vec![self.parent.layout_main(cx)];
            if let Some(children) = self.children.as_mut() {
                nodes.push(children.layout_main(cx))
            }
            nodes
        })
    }

    fn debug_name(&self) -> std::borrow::Cow<'static, str> {
        "Tree Simple".into()
    }

    fn paint(&mut self, cx: &mut crate::context::PaintCx) {
        self.parent.paint_main(cx);
        if let Some(children) = self.children.as_mut() {
            children.paint_main(cx);
        }
    }

    fn update(
        &mut self,
        cx: &mut crate::context::UpdateCx,
        state: Box<dyn std::any::Any>,
    ) -> crate::view::ChangeFlags {
        crate::view::ChangeFlags::empty()
    }
}

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
pub struct ReactiveTree<T>
where
    T: std::fmt::Debug + 'static,
{
    scope: Scope,
    root: Id,
    nodes: RwSignal<HashMap<Id, RwSignal<T>>>,
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
        nodes.insert(root, create_rw_signal(scope, data));
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
            nodes.insert(child, create_rw_signal(self.scope, data));
        });

        self.nodes
            .get_untracked()
            .get(&parent)
            .unwrap()
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
                    nodes.get_untracked().get(&id).unwrap().get()
                }),
            }))
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

// impl <T> IntoIterator for ConcreteTreeNode<>

// impl<T> IntoIterator for ReactiveTree<T>
// where
//     T: Copy + Clone + 'static,
// {
//     type Item = TreeNodeIterItem<T>;
//     type IntoIter = TreeNodeIter<T>;

//     fn into_iter(self) -> Self::IntoIter {
//         let first_child = self.root_children().get_untracked().first().map(|id| *id);
//         TreeNodeIter {
//             cur: first_child,
//             parent: self.root,
//             scope: self.scope,
//             tree: self,
//         }
//     }
// }

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
        println!("created iter with {:?}", first_child);
        TreeNodeIter {
            cur: first_child,
            parent: self.id,
            scope: self.scope,
            tree: self.tree,
        }
    }
}

impl<T> SuperFuckingBasicTreeNode for ConcreteTreeNode<T>
where
    T: Debug + Clone + Hash + Eq + 'static,
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
            label(move || format!("Node with id {:?}", x.get().id))
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
    S: SuperFuckingBasicTreeNode<Item = N, K = K> + 'static,
    N: SignalGet<S>,
    K: Debug + Hash + Eq + 'static,
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
                move |item| build_fucking_simple_tree(item.get()),
            ))
        } else {
            None
        }
    };

    tree_view(tree_node.node(), parent, children)
        .style(|| Style::BASE.flex_direction(FlexDirection::Column))
}
