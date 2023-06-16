use leptos_reactive::{
    create_rw_signal, ReadSignal, RwSignal, Scope, SignalGet, SignalGetUntracked, SignalUpdate,
};
use taffy::style::{FlexDirection, LengthPercentage, LengthPercentageAuto};

use crate::{id::Id, style::Style, view::View, ViewContext};

use super::{
    label, list, scroll,
    tree_builder::{tree_view, Children, Node, TreeView},
    virtual_list, Decorators, Label, List, VirtualList, VirtualListDirection, VirtualListItemSize,
    VirtualListVector,
};
use std::{collections::HashMap, hash::Hash, rc::Rc};

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
    fn key(&self) -> Self::KeyFn;
    fn view_fn(&self) -> Self::ViewFn;
}

impl<N, T, V, I> TreeNode<T, V, I> for Rc<N>
where
    N: TreeNode<T, V, I>,
    I: IntoIterator + 'static,
    V: View + 'static,
    T: 'static,
{
    type Children = N::Children;
    type Item = N::Item;
    type K = N::K;
    type KeyFn = N::KeyFn;
    type ViewFn = N::ViewFn;

    fn children(&self) -> Self::Children {
        self.as_ref().children()
    }

    fn has_children(&self) -> bool {
        self.as_ref().has_children()
    }

    fn key(&self) -> Self::KeyFn {
        self.as_ref().key()
    }

    fn node(&self) -> Self::Item {
        self.as_ref().node()
    }

    fn view_fn(&self) -> Self::ViewFn {
        self.as_ref().view_fn()
    }
}

// want a data structure that is a tree of signals

#[derive(Clone)]
pub struct ReactiveTree<T: 'static> {
    scope: Scope,
    root: Id,
    nodes: RwSignal<HashMap<Id, RwSignal<T>>>,
    children: RwSignal<HashMap<Id, RwSignal<Vec<Id>>>>,
}

impl<T> ReactiveTree<T> {
    pub fn new(scope: Scope, root: Id, data: T) -> Self {
        let nodes = HashMap::new();
        Self {
            scope,
            root,
            nodes: create_rw_signal(scope, nodes),
            children: create_rw_signal(scope, HashMap::new()),
        }
    }

    pub fn insert_child(&self, parent: Id, child: Id, data: T) {
        let mut children = self.children.get_untracked();

        if let Some(parent_children) = children.get_mut(&parent) {
            parent_children.update(|parent_children| parent_children.push(child));
        } else {
            children.insert(parent, create_rw_signal(self.scope, vec![child]));
        };

        self.nodes.update(|nodes| {
            nodes.insert(child, create_rw_signal(self.scope, data));
        });
    }

    fn direct_children(&self) -> RwSignal<Vec<Id>> {
        // we should be gucci to unwrap here
        *self.children.get_untracked().get(&self.root).unwrap()
    }

    fn next_child(&self, child: Id) -> Option<Id> {
        let direct_children = self.direct_children().get_untracked();
        let index = direct_children.iter().position(|c| *c == child);

        index.and_then(|i| direct_children.get(i + 1).map(|id| *id))
    }
}

#[derive(Clone)]
pub struct ReactiveTreeChildIter<T>
where
    T: Clone + 'static,
{
    tree: ReactiveTree<T>,
    cur: Option<Id>,
}

impl<T> Iterator for ReactiveTreeChildIter<T>
where
    T: Clone,
{
    type Item = ReactiveTree<T>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cur) = self.cur {
            self.cur = self.tree.next_child(cur);
            Some(ReactiveTree {
                scope: self.tree.scope,
                root: cur,
                children: self.tree.children,
                nodes: self.tree.nodes,
            })
        } else {
            None
        }
    }
}

impl<T> IntoIterator for ReactiveTree<T>
where
    T: Clone,
{
    type Item = ReactiveTree<T>;
    type IntoIter = ReactiveTreeChildIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        ReactiveTreeChildIter {
            tree: self.clone(),
            cur: self.direct_children().get_untracked().first().map(|id| *id),
        }
    }
}

// impl<T> TreeNode<T, Label, ReactiveTree<T>> for ReactiveTree<T>
// where
//     T: Clone,
// {
//     type Children = RwSignal<ReactiveTree<T>>;
//     type Item = ReactiveTree<ReactiveTree<T>>;
//     type K = Id;
//     type KeyFn = Box<dyn Fn(&Self::Item) -> Self::K>;
//     type ViewFn = Box<dyn Fn(&Self::Item) -> Label>;
//     type Value = RwSignal<T>;

//     fn has_children(&self) -> bool {
//         self.direct_children().get_untracked().len() > 0
//     }

//     fn children(&self) -> Self::Children {
//         create_rw_signal(self.scope, self.into)
//     }

//     fn key(&self) -> Self::KeyFn {
//         Box::new(|item| item.root)
//     }

//     fn node(&self) -> Self::Item {
//         self.clone()
//     }

//     fn view_fn(&self) -> Self::ViewFn {
//         Box::new(|item| Label::new(format!("Item: {}", item.root)))
//     }
// }

// needs a way to tell me to build children
// need to get a type that is is a closure that returns IntoIter<T> where T: SignalGet
// need a type that returns a closure that is |value: T|  -> V where V: View, T: SignalGet

// i need a method to get something that can be turned into an iterator of children (wrapped in a signal). This iterator should return the same type as the node
// i need a method that can can return a closure
// i need a method that can return the current node

trait SuperFuckingBasicTreeNode {
    type View: View;
    type I: IntoIterator<Item = Self>;
    type Children: SignalGet<Self::I>;
    type K: Hash + Eq + 'static;
    type KeyFn: Fn(&Self) -> Self::K + 'static;
    type ViewFn: Fn(&Self) -> Self::View;

    fn has_children(&self) -> bool;
    fn children(&self) -> Self::Children;
    fn key_fn(&self) -> Self::KeyFn;
    fn view_fn(&self) -> Self::ViewFn;
}

impl<T> SuperFuckingBasicTreeNode for ReactiveTree<T>
where
    T: Clone,
{
    type View = Label;
    type Children = ReadSignal<Self::I>;
    type I = ReactiveTree<T>;
    type K = Id;
    type KeyFn = Box<dyn Fn(&Self) -> Id>;
    type ViewFn = Box<dyn Fn(&Self) -> Label>;

    fn has_children(&self) -> bool {
        self.direct_children().get_untracked().len() > 0
    }

    fn children(&self) -> Self::Children {
        create_rw_signal(self.scope, self.clone()).read_only()
    }

    fn key_fn(&self) -> Self::KeyFn {
        Box::new(|node| node.root)
    }

    fn view_fn(&self) -> Self::ViewFn {
        Box::new(|node| {
            let root_id = node.root;
            label(move || format!("Node with id {:?}", root_id))
        })
    }
}

pub struct NeverIterate<T>(std::marker::PhantomData<T>);

impl<T> IntoIterator for NeverIterate<T> {
    type IntoIter = std::iter::Empty<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::empty()
    }
}

fn build_fucking_simple_tree<T, V, S, I>(tree_node: S) -> TreeView<S::Item, V>
where
    S: TreeNode<T, V, I, Item = S> + Copy + 'static,
    V: View + 'static,
    I: IntoIterator<Item = S> + 'static,
    T: 'static,
{
    let parent = Node::new(tree_node.view_fn());
    let children = Children::new(
        move || tree_node.children().get(),
        tree_node.key(),
        move || {
            Box::new(move |x| {
                if tree_node.has_children() {
                    build_fucking_simple_tree(x)
                } else {
                    let parent = Node::new(tree_node.view_fn());
                    tree_view::<_, _, NeverIterate<_>, S::K>(x, parent, None)
                }
            })
        },
    );

    tree_view(tree_node.node(), parent, Some(children))
        .style(|| Style::BASE.flex_direction(FlexDirection::Column))
}
