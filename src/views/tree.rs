use crate::{id::Id, view::View, ViewContext};

use super::{
    scroll, virtual_list, VirtualList, VirtualListDirection, VirtualListItemSize, VirtualListVector,
};
use std::hash::Hash;

// a tree is a list of tree items
// a tree item can either return a tree or a leaf

// a tree is a a view made up of indented lists
pub enum TreeData<T, IF, KF, PVF, CVF, L>
where
    CVF: Fn(T) -> L + 'static,
{
    Leaf(LeafData<T, CVF>),
    Parent(ParentData<T, IF, KF, PVF>),
}

pub struct LeafData<T, CVF> {
    item: T,
    view_fn: Box<CVF>,
}

pub struct ParentData<T, IF, KF, PVF> {
    pub parent_view_fn: Box<PVF>,

    pub item_size: VirtualListItemSize<T>,
    pub each_fn: Box<IF>,
    pub key_fn: Box<KF>,
    pub child_view_fn: Box<PVF>,
}

pub struct Tree<T, CFV, V, L>
where
    T: 'static,
    CFV: Fn(T) -> V + 'static,
    V: View + 'static,
    L: View,
{
    id: Id,
    child: TreeChild<T, CFV, V, L>,
}

enum TreeChild<T, CVF, V, L>
where
    T: 'static,
    CVF: Fn(T) -> V + 'static,
    V: View + 'static,
    L: View,
{
    Leaf(L),
    Branch(VirtualList<V, CVF, T>),
}

pub fn tree<T, IF, I, KF, K, PVF, CVF, V, L>(
    tree_data: TreeData<T, IF, KF, PVF, CVF, L>,
) -> Tree<T, PVF, V, L>
where
    K: Hash + Eq + 'static,
    I: VirtualListVector<T> + 'static,
    IF: Fn() -> I + 'static,
    KF: Fn(&T) -> K + 'static,
    PVF: Fn(T) -> V + 'static,
    CVF: Fn(T) -> L + 'static,
    T: 'static,
    V: View,
    L: View,
{
    let id = ViewContext::get_current().new_id();
    Tree {
        id,
        child: match tree_data {
            TreeData::Parent(branch) => TreeChild::Branch(virtual_list(
                VirtualListDirection::Vertical,
                VirtualListItemSize::Fixed(Box::new(|| 32.0)),
                branch.each_fn,
                branch.key_fn,
                *branch.child_view_fn,
            )),
            TreeData::Leaf(leaf) => TreeChild::Leaf((leaf.view_fn)(leaf.item)),
        },
    }
}

impl<T, PVF, V, L> View for Tree<T, PVF, V, L>
where
    T: 'static,
    PVF: Fn(T) -> V + 'static,
    V: View,
    L: View,
{
    fn id(&self) -> Id {
        self.id
    }

    fn debug_name(&self) -> std::borrow::Cow<'static, str> {
        "Tree".into()
    }

    fn event(
        &mut self,
        cx: &mut crate::context::EventCx,
        id_path: Option<&[Id]>,
        event: crate::event::Event,
    ) -> bool {
        match &mut self.child {
            TreeChild::Leaf(leaf) => leaf.event(cx, id_path, event),
            TreeChild::Branch(branch) => branch.event(cx, id_path, event),
        }
    }

    fn layout(&mut self, cx: &mut crate::context::LayoutCx) -> taffy::prelude::Node {
        match &mut self.child {
            TreeChild::Leaf(leaf) => leaf.layout(cx),
            TreeChild::Branch(branch) => branch.layout(cx),
        }
    }

    fn paint(&mut self, cx: &mut crate::context::PaintCx) {
        match &mut self.child {
            TreeChild::Leaf(leaf) => leaf.paint(cx),
            TreeChild::Branch(branch) => branch.paint(cx),
        }
    }

    fn update(
        &mut self,
        cx: &mut crate::context::UpdateCx,
        state: Box<dyn std::any::Any>,
    ) -> crate::view::ChangeFlags {
        match &mut self.child {
            TreeChild::Leaf(leaf) => leaf.update(cx, state),
            TreeChild::Branch(branch) => branch.update(cx, state),
        }
    }

    fn children(&mut self) -> Vec<&mut dyn View> {
        match &mut self.child {
            TreeChild::Leaf(leaf) => vec![leaf],
            TreeChild::Branch(branch) => vec![branch],
        }
    }

    fn child(&mut self, id: Id) -> Option<&mut dyn View> {
        match &mut self.child {
            TreeChild::Leaf(leaf) => {
                if leaf.id() == id {
                    Some(leaf)
                } else {
                    None
                }
            }
            TreeChild::Branch(branch) => {
                if branch.id() == id {
                    Some(branch)
                } else {
                    None
                }
            }
        }
    }
}
