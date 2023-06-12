use taffy::style::{LengthPercentage, LengthPercentageAuto};

use crate::{id::Id, style::Style, view::View, ViewContext};

use super::{
    list, scroll, virtual_list, Decorators, List, VirtualList, VirtualListDirection,
    VirtualListItemSize, VirtualListVector,
};
use std::{hash::Hash, rc::Rc};

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
